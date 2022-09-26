use crate::{
	chain_spec,
	cli::{Cli, Subcommand},
	service::{self, db_config_dir, IdentifyVariant},
};
use clap::Parser;
use fc_db::frontier_database_dir;
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use sc_service::{DatabaseSource, PartialComponents};

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Meta Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/DeFiCh/metachain/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2022
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			#[cfg(feature = "meta-native")]
			"meta-dev" => Box::new(chain_spec::meta::development_config()?),
			#[cfg(feature = "meta-native")]
			"meta-local" => Box::new(chain_spec::meta::local_testnet_config()?),
			#[cfg(feature = "birthday-native")]
			"birthday-dev" => Box::new(chain_spec::birthday::development_config()?),
			#[cfg(feature = "birthday-native")]
			"" | "birthday-local" => Box::new(chain_spec::birthday::local_testnet_config()?),
			path => Box::new(chain_spec::ChainSpec::from_json_file(
				std::path::PathBuf::from(path),
			)?),
		})
	}

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		match spec {
			#[cfg(feature = "meta-native")]
			spec if spec.is_meta() => return &meta_runtime::VERSION,
			#[cfg(feature = "birthday-native")]
			spec if spec.is_birthday() => return &birthday_runtime::VERSION,
			_ => panic!("invalid chain spec"),
		}
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::parse();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) =
					service::new_chain_ops(&mut config, &cli)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config, &cli)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config, &cli)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) =
					service::new_chain_ops(&mut config, &cli)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				// Remove Frontier offchain db
				let db_config_dir = db_config_dir(&config);
				let frontier_database_config = match config.database {
					DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
						path: frontier_database_dir(&db_config_dir, "db"),
						cache_size: 0,
					},
					DatabaseSource::ParityDb { .. } => DatabaseSource::ParityDb {
						path: frontier_database_dir(&db_config_dir, "paritydb"),
					},
					_ => {
						return Err(format!("Cannot purge `{:?}` database", config.database).into())
					}
				};
				cmd.run(frontier_database_config)?;
				cmd.run(config.database)
			})
		}
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;
			match chain_spec {
				#[cfg(feature = "meta-native")]
				spec if spec.is_meta() => runner.async_run(|mut config| {
					let params = service::new_partial::<
						meta_runtime::RuntimeApi,
						service::MetaExecutor,
					>(&mut config, &cli)?;

					Ok((
						cmd.run(params.client, params.backend, None),
						params.task_manager,
					))
				}),
				#[cfg(feature = "birthday-native")]
				spec if spec.is_birthday() => runner.async_run(|mut config| {
					let params = service::new_partial::<
						birthday_runtime::RuntimeApi,
						service::BirthdayExecutor,
					>(&mut config, &cli)?;

					Ok((
						cmd.run(params.client, params.backend, None),
						params.task_manager,
					))
				}),
				_ => panic!("invalid chain spec"),
			}
		}
		Some(Subcommand::FrontierDb(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;
			match chain_spec {
				#[cfg(feature = "meta-native")]
				spec if spec.is_meta() => runner.sync_run(|config| {
					let PartialComponents { client, other, .. } = service::new_partial::<
						meta_runtime::RuntimeApi,
						service::MetaExecutor,
					>(&config, &cli)?;
					let frontier_backend = other.2;
					cmd.run::<_, meta_primitives::Block>(client, frontier_backend)
				}),
				#[cfg(feature = "birthday-native")]
				spec if spec.is_birthday() => runner.sync_run(|config| {
					let PartialComponents { client, other, .. } = service::new_partial::<
						birthday_runtime::RuntimeApi,
						service::BirthdayExecutor,
					>(&config, &cli)?;
					let frontier_backend = other.2;
					cmd.run::<_, meta_primitives::Block>(client, frontier_backend)
				}),
				_ => panic!("invalid chain spec"),
			}
		}
		None => {
			let runner = cli.create_runner(&cli.run.base)?;
			let chain_spec = &runner.config().chain_spec;
			let is_meta = chain_spec.is_meta();

			runner.run_node_until_exit(|config| async move {
				if is_meta {
					service::new_full::<meta_runtime::RuntimeApi, service::MetaExecutor>(
						config, &cli,
					)
					.map_err(sc_cli::Error::Service)
				} else {
					service::new_full::<birthday_runtime::RuntimeApi, service::BirthdayExecutor>(
						config, &cli,
					)
					.map_err(sc_cli::Error::Service)
				}
			})
		}
	}
}
