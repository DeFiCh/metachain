//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.
#![allow(clippy::needless_borrow)]
use std::{
	cell::RefCell,
	collections::BTreeMap,
	path::PathBuf,
	sync::{Arc, Mutex},
	time::Duration,
};

use futures::{future, StreamExt};
// Substrate
use sc_cli::SubstrateCli;
use sc_client_api::BlockchainEvents;
use sc_executor::NativeElseWasmExecutor;
use sc_keystore::LocalKeystore;
use sc_service::{
	error::Error as ServiceError, BasePath, Configuration, PartialComponents, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_core::U256;
use sp_inherents::{InherentData, InherentIdentifier};
// Frontier
use fc_consensus::FrontierBlockImport;
use fc_db::Backend as FrontierBackend;
use fc_mapping_sync::{MappingSyncWorker, SyncStrategy};
use fc_rpc::{EthTask, OverrideHandle};
use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
// Runtime
use crate::cli::Cli;
use crate::cli::Sealing;
use meta_primitives::Block;
// TODO(canonbrother): use match
use meta_runtime::RuntimeApi;

// Our native executor instance.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
	/// Only enable the benchmarking host functions when we actually want to benchmark.
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	/// Otherwise we only use the default Substrate host functions.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		meta_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		meta_runtime::native_version()
	}
}

pub type FullClient =
	sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub type ConsensusResult = (
	FrontierBlockImport<Block, Arc<FullClient>, FullClient>,
	Sealing,
);

pub(crate) fn db_config_dir(config: &Configuration) -> PathBuf {
	config
		.base_path
		.as_ref()
		.map(|base_path| base_path.config_dir(config.chain_spec.id()))
		.unwrap_or_else(|| {
			BasePath::from_project("", "", &Cli::executable_name())
				.config_dir(config.chain_spec.id())
		})
}

/// Returns most parts of a service. Not enough to run a full chain,
/// But enough to perform chain operations like purge-chain
pub fn new_partial(
	config: &Configuration,
	cli: &Cli,
) -> Result<
	PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block, FullClient>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(
			Option<Telemetry>,
			ConsensusResult,
			Arc<FrontierBackend<Block>>,
			Option<FilterPool>,
			(FeeHistoryCache, FeeHistoryCacheLimit),
		),
	>,
	ServiceError,
> {
	if config.keystore_remote.is_some() {
		return Err(ServiceError::Other(
			"Remote Keystores are not supported.".to_string(),
		));
	}

	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = NativeElseWasmExecutor::<ExecutorDispatch>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
		config.runtime_cache_size,
	);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager
			.spawn_handle()
			.spawn("telemetry", None, worker.run());
		telemetry
	});

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let frontier_backend = Arc::new(FrontierBackend::open(
		&config.database,
		&db_config_dir(config),
	)?);
	let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));
	let fee_history_cache: FeeHistoryCache = Arc::new(Mutex::new(BTreeMap::new()));
	let fee_history_cache_limit: FeeHistoryCacheLimit = cli.run.fee_history_limit;

	let sealing = cli.run.sealing;

	let frontier_block_import =
		FrontierBlockImport::new(client.clone(), client.clone(), frontier_backend.clone());

	let import_queue = sc_consensus_manual_seal::import_queue(
		Box::new(client.clone()),
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
	);

	Ok(PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (
			telemetry,
			(frontier_block_import, sealing),
			frontier_backend,
			filter_pool,
			(fee_history_cache, fee_history_cache_limit),
		),
	})
}

fn remote_keystore(_url: &str) -> Result<Arc<LocalKeystore>, &'static str> {
	// FIXME: here would the concrete keystore be built,
	//        must return a concrete type (NOT `LocalKeystore`) that
	//        implements `CryptoStore` and `SyncCryptoStore`
	Err("Remote Keystore not supported.")
}

/// Builds a new service for a full client.
pub fn new_full(mut config: Configuration, cli: &Cli) -> Result<TaskManager, ServiceError> {
	// Use ethereum style for subscription ids
	config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));

	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		mut keystore_container,
		select_chain,
		transaction_pool,
		other:
			(
				mut telemetry,
				consensus_result,
				frontier_backend,
				filter_pool,
				(fee_history_cache, fee_history_cache_limit),
			),
	} = new_partial(&config, cli)?;

	if let Some(url) = &config.keystore_remote {
		match remote_keystore(url) {
			Ok(k) => keystore_container.set_remote_keystore(k),
			Err(e) => {
				return Err(ServiceError::Other(format!(
					"Error hooking up remote keystore for {}: {}",
					url, e
				)))
			}
		};
	}

	let (network, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let role = config.role.clone();
	let prometheus_registry = config.prometheus_registry().cloned();
	let overrides = crate::rpc::overrides_handle(client.clone());
	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		overrides.clone(),
		50,
		50,
		prometheus_registry.clone(),
	));
	// Channel for the rpc handler to communicate with the authorship task.
	let (command_sink, commands_stream) = futures::channel::mpsc::channel(1000);

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let is_authority = role.is_authority();
		let enable_dev_signer = cli.run.enable_dev_signer;
		let network = network.clone();
		let filter_pool = filter_pool.clone();
		let frontier_backend = frontier_backend.clone();
		let overrides = overrides.clone();
		let fee_history_cache = fee_history_cache.clone();
		let max_past_logs = cli.run.max_past_logs;

		Box::new(move |deny_unsafe, subscription_task_executor| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				deny_unsafe,
				is_authority,
				enable_dev_signer,
				network: network.clone(),
				filter_pool: filter_pool.clone(),
				backend: frontier_backend.clone(),
				max_past_logs,
				fee_history_cache: fee_history_cache.clone(),
				fee_history_cache_limit,
				overrides: overrides.clone(),
				block_data_cache: block_data_cache.clone(),
				command_sink: Some(command_sink.clone()),
			};

			crate::rpc::create_full(deps, subscription_task_executor).map_err(Into::into)
		})
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network,
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_builder: rpc_extensions_builder,
		backend: backend.clone(),
		system_rpc_tx,
		config,
		telemetry: telemetry.as_mut(),
	})?;

	spawn_frontier_tasks(
		&task_manager,
		client.clone(),
		backend,
		frontier_backend,
		filter_pool,
		overrides,
		fee_history_cache,
		fee_history_cache_limit,
	);

	if role.is_authority() {
		let env = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let (block_import, sealing) = consensus_result;

		const INHERENT_IDENTIFIER: InherentIdentifier = *b"timstap0";
		thread_local!(static TIMESTAMP: RefCell<u64> = RefCell::new(0));

		/// Provide a mock duration starting at 0 in millisecond for timestamp inherent.
		struct MockTimestampInherentDataProvider;

		#[async_trait::async_trait]
		impl sp_inherents::InherentDataProvider for MockTimestampInherentDataProvider {
			fn provide_inherent_data(
				&self,
				inherent_data: &mut InherentData,
			) -> Result<(), sp_inherents::Error> {
				TIMESTAMP.with(|x| {
					*x.borrow_mut() += meta_runtime::SLOT_DURATION;
					inherent_data.put_data(INHERENT_IDENTIFIER, &*x.borrow())
				})
			}

			async fn try_handle_error(
				&self,
				_identifier: &InherentIdentifier,
				_error: &[u8],
			) -> Option<Result<(), sp_inherents::Error>> {
				// The pallet never reports error.
				None
			}
		}

		let target_gas_price = cli.run.target_gas_price;
		let create_inherent_data_providers = move |_, ()| async move {
			let mock_timestamp = MockTimestampInherentDataProvider;
			let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
			Ok((mock_timestamp, dynamic_fee))
		};

		let manual_seal = match sealing {
			Sealing::Manual => future::Either::Left(sc_consensus_manual_seal::run_manual_seal(
				sc_consensus_manual_seal::ManualSealParams {
					block_import,
					env,
					client,
					pool: transaction_pool,
					commands_stream,
					select_chain,
					consensus_data_provider: None,
					create_inherent_data_providers,
				},
			)),
			Sealing::Instant => future::Either::Right(sc_consensus_manual_seal::run_instant_seal(
				sc_consensus_manual_seal::InstantSealParams {
					block_import,
					env,
					client,
					pool: transaction_pool,
					select_chain,
					consensus_data_provider: None,
					create_inherent_data_providers,
				},
			)),
		};
		// we spawn the future on a background thread managed by service.
		task_manager
			.spawn_essential_handle()
			.spawn_blocking("manual-seal", None, manual_seal);
	};

	log::info!("Manual Seal Ready");

	network_starter.start_network();
	Ok(task_manager)
}

fn spawn_frontier_tasks(
	task_manager: &TaskManager,
	client: Arc<FullClient>,
	backend: Arc<FullBackend>,
	frontier_backend: Arc<FrontierBackend<Block>>,
	filter_pool: Option<FilterPool>,
	overrides: Arc<OverrideHandle<Block>>,
	fee_history_cache: FeeHistoryCache,
	fee_history_cache_limit: FeeHistoryCacheLimit,
) {
	task_manager.spawn_essential_handle().spawn(
		"frontier-mapping-sync-worker",
		None,
		MappingSyncWorker::new(
			client.import_notification_stream(),
			Duration::new(6, 0),
			client.clone(),
			backend,
			frontier_backend,
			3,
			0,
			SyncStrategy::Normal,
		)
		.for_each(|()| future::ready(())),
	);

	// Spawn Frontier EthFilterApi maintenance task.
	if let Some(filter_pool) = filter_pool {
		// Each filter is allowed to stay in the pool for 100 blocks.
		const FILTER_RETAIN_THRESHOLD: u64 = 100;
		task_manager.spawn_essential_handle().spawn(
			"frontier-filter-pool",
			None,
			EthTask::filter_pool_task(client.clone(), filter_pool, FILTER_RETAIN_THRESHOLD),
		);
	}

	// Spawn Frontier FeeHistory cache maintenance task.
	task_manager.spawn_essential_handle().spawn(
		"frontier-fee-history",
		None,
		EthTask::fee_history_task(
			client,
			overrides,
			fee_history_cache,
			fee_history_cache_limit,
		),
	);
}
