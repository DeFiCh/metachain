use meta_runtime::{
	AccountId, BalancesConfig, GenesisConfig, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use sp_core::{sr25519, Pair};
use sp_runtime::traits::{IdentifyAccount, Verify};

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPair: Pair>(seed: &str) -> TPair::Public {
	TPair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn account_id_from_seed<TPair: Pair>(seed: &str) -> AccountId
where
	AccountPublic: From<TPair::Public>,
{
	AccountPublic::from(get_from_seed::<TPair>(seed)).into_account()
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		sc_service::ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Root Key
				account_id_from_seed::<sr25519::Pair>("Alice"),
				// Endowed Accounts
				vec![
					account_id_from_seed::<sr25519::Pair>("Alice"),
					account_id_from_seed::<sr25519::Pair>("Bob"),
					account_id_from_seed::<sr25519::Pair>("Alice//stash"),
					account_id_from_seed::<sr25519::Pair>("Bob//stash"),
				],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		sc_service::ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				account_id_from_seed::<sr25519::Pair>("Alice"),
				vec![
					account_id_from_seed::<sr25519::Pair>("Alice"),
					account_id_from_seed::<sr25519::Pair>("Bob"),
					account_id_from_seed::<sr25519::Pair>("Charlie"),
					account_id_from_seed::<sr25519::Pair>("Dave"),
					account_id_from_seed::<sr25519::Pair>("Eve"),
					account_id_from_seed::<sr25519::Pair>("Ferdie"),
					account_id_from_seed::<sr25519::Pair>("Alice//stash"),
					account_id_from_seed::<sr25519::Pair>("Bob//stash"),
					account_id_from_seed::<sr25519::Pair>("Charlie//stash"),
					account_id_from_seed::<sr25519::Pair>("Dave//stash"),
					account_id_from_seed::<sr25519::Pair>("Eve//stash"),
					account_id_from_seed::<sr25519::Pair>("Ferdie//stash"),
				],
			)
		},
		vec![],
		None,
		None,
		None,
		None,
	))
}

/// Helper function to build a genesis configuration
pub fn testnet_genesis(
	wasm_binary: &[u8],
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
	}
}
