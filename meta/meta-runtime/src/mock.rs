#![cfg(test)]
use super::*;

use core::str::FromStr;
use fp_evm::{GenesisAccount, Precompile};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64, Everything, GenesisBuild},
};
use sp_core::H160;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use std::collections::BTreeMap;

use pallet_evm::{
	EnsureAddressNever, EnsureAddressRoot, IdentityAddressMapping, PrecompileHandle,
	PrecompileResult, PrecompileSet,
};

pub type AccountId = H160;
pub type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const INITIAL_BALANCE: Balance = 1_000_000_000_000_000;

pub fn alice() -> H160 {
	// H160::from_str("efdc17c993a573e5933f230cbA79073Cf71c1D01").unwrap() // random
	// H160::from_str("1cbd2d43530a44705ad088af313e18f80b53ef16").unwrap() // Ferdie
	// H160::from_str("e659a7a1628cdd93febc04a4e0646ea20e9f5f0c").unwrap() // Eve
	// H160::from_str("306721211d5404bd9da88e0204360a1a9ab8b87c").unwrap() // Dave
	// H160::from_str("90b5ab205c6974c9ea841be688864633dc9ca8a3").unwrap() // Charlie
	// H160::from_str("fe65717dad0447d715f660a0a58411de509b42e6").unwrap() // Bob/stash
	// H160::from_str("be5ddb1579b72e84524fc29e78609e3caf42e85a").unwrap() // Alice/stash
	// H160::from_str("8eaf04151687736326c9fea17e25fc5287613693").unwrap() // Bob
	// H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558").unwrap() // Alice
	// H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b").unwrap() // ci
	H160::from_str("1000000000000000000000000000000000000001").unwrap()
	// H160::default() // root
}

construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system,
			Balances: pallet_balances,
			Timestamp: pallet_timestamp,
			EVM: pallet_evm,
		}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(Weight::from_ref_time(1024).set_proof_size(u64::MAX));
}
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = ();
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1000>;
	type WeightInfo = ();
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		// Return some meaningful gas price and weight
		(1_000_000_000u128.into(), Weight::from_ref_time(7u64))
	}
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub WeightPerGas: Weight = Weight::from_ref_time(20_000);
	pub MockPrecompiles: MockPrecompileSet = MockPrecompileSet;
}
impl pallet_evm::Config for Test {
	// type FeeCalculator = FixedGasPrice; // BalanceLow err
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = MockPrecompileSet;
	type PrecompilesValue = MockPrecompiles;
	type ChainId = ();
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = ();
	type FindAuthor = ();
}

/// Example PrecompileSet with only Identity precompile.
pub struct MockPrecompileSet;

impl PrecompileSet for MockPrecompileSet {
	/// Tries to execute a precompile in the precompile set.
	/// If the provided address is not a precompile, returns None.
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		let address = handle.code_address();

		if address == H160::from_low_u64_be(1) {
			return Some(pallet_evm_precompile_simple::Identity::execute(handle));
		}

		None
	}

	/// Check if the given address is a precompile. Should only be called to
	/// perform the check while not executing the precompile afterward, since
	/// `execute` already performs a check internally.
	fn is_precompile(&self, address: H160) -> bool {
		address == H160::from_low_u64_be(1)
	}
}

#[derive(Default)]
pub(crate) struct ExtBuilder {
	// Accounts endowed with balances
	balances: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Test ExtBuilder setup successfully");
		pallet_balances::GenesisConfig::<Test> {
			balances: vec![(H160::default(), INITIAL_BALANCE)],
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");
		let mut accounts = BTreeMap::<H160, GenesisAccount>::new();
		accounts.insert(
			alice(),
			GenesisAccount {
				nonce: U256::from("1"),
				balance: U256::from(INITIAL_BALANCE),
				storage: Default::default(),
				code: vec![],
			},
		);
		GenesisBuild::<Test>::assimilate_storage(&pallet_evm::GenesisConfig { accounts }, &mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
