use super::*;
use crate as utility;

use sc_consensus::{
	import_single_block, BasicQueue, BlockImportError, BlockImportStatus, ImportedAux,
	IncomingBlock, 
};

use sp_consensus::BlockOrigin;
use libp2p::{build_multiaddr, PeerId};

use sp_runtime::generic::BlockId;
use substrate_test_runtime_client::{
	self,
	prelude::*,
	runtime::Hash,
};
use futures::executor::block_on;
use frame_support::{
	assert_err_ignore_postinfo, assert_noop, assert_ok,
	dispatch::{DispatchError, DispatchErrorWithPostInfo, Dispatchable},
	parameter_types, storage,
	traits::{ConstU32, ConstU64, Contains},
	weights::{Pays, Weight},
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
// example module to test behaviors.
#[frame_support::pallet]
pub mod example {
	use frame_support::{dispatch::WithPostDispatchInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(*_weight)]
		pub fn noop(_origin: OriginFor<T>, _weight: Weight) -> DispatchResult {
			Ok(())
		}

		#[pallet::weight(*_start_weight)]
		pub fn foobar(
			origin: OriginFor<T>,
			err: bool,
			_start_weight: Weight,
			end_weight: Option<Weight>,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			if err {
				let error: DispatchError = "The cake is a lie.".into();
				if let Some(weight) = end_weight {
					Err(error.with_weight(weight))
				} else {
					Err(error)?
				}
			} else {
				Ok(end_weight.into())
			}
		}

		#[pallet::weight(0)]
		pub fn big_variant(_origin: OriginFor<T>, _arg: [u8; 400]) -> DispatchResult {
			Ok(())
		}
	}
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Utility: utility::{Pallet, Call, Event},
		Example: example::{Pallet, Call},
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(Weight::max_value());
}

impl frame_system::Config for Test {
	type BaseCallFilter = TestBaseCallFilter;
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const MultisigDepositBase: u64 = 1;
	pub const MultisigDepositFactor: u64 = 1;
	pub const MaxSignatories: u16 = 3;
}

impl example::Config for Test {}

pub struct TestBaseCallFilter;
impl Contains<Call> for TestBaseCallFilter {
	fn contains(c: &Call) -> bool {
		match *c {
			// Transfer works. Use `transfer_keep_alive` for a call that doesn't pass the filter.
			Call::Balances(pallet_balances::Call::transfer { .. }) => true,
			Call::Utility(_) => true,
			// For benchmarking, this acts as a noop call
			Call::System(frame_system::Call::remark { .. }) => true,
			// For tests
			Call::Example(_) => true,
			_ => false,
		}
	}
}
impl Config for Test {
	type Event = Event;
	type Call = Call;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}
type ExampleCall = example::Call<Test>;
type UtilityCall = crate::Call<Test>;

use frame_system::Call as SystemCall;
use pallet_balances::{Call as BalancesCall, Error as BalancesError};

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10), (2, 10), (3, 10), (4, 10), (5, 2)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn call_transfer(dest: u64, value: u64) -> Call {
	Call::Balances(BalancesCall::transfer { dest, value })
}

fn call_foobar(err: bool, start_weight: u64, end_weight: Option<u64>) -> Call {
	Call::Example(ExampleCall::foobar { err, start_weight, end_weight })
}


#[test]
fn as_derivative_works() {
	new_test_ext().execute_with(|| {
		let sub_1_0 = Utility::derivative_account_id(1, 0);
		assert_ok!(Balances::transfer(Origin::signed(1), sub_1_0, 5));
		assert_err_ignore_postinfo!(
			Utility::as_derivative(Origin::signed(1), 1, Box::new(call_transfer(6, 3)),),
			BalancesError::<Test, _>::InsufficientBalance
		);
		assert_ok!(Utility::as_derivative(Origin::signed(1), 0, Box::new(call_transfer(2, 3)),));
		assert_eq!(Balances::free_balance(sub_1_0), 2);
		assert_eq!(Balances::free_balance(2), 13);
	});
}

// #[test]
// fn test_should_work() {

// }

#[test]
fn batch_all_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
		assert_ok!(Utility::batch_all(
			Origin::signed(1),
			vec![call_transfer(2, 5), call_transfer(2, 5)]
		),);
		assert_eq!(Balances::free_balance(1), 0);
		assert_eq!(Balances::free_balance(2), 20);
	});
}

#[test]
fn batch_all_revert() {
	new_test_ext().execute_with(|| {
		let call = call_transfer(2, 5);
		let info = call.get_dispatch_info();

		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
		let batch_all_calls = Call::Utility(crate::Call::<Test>::batch_all {
			calls: vec![call_transfer(2, 5), call_transfer(2, 10), call_transfer(2, 5)],
		});
		assert_noop!(
			batch_all_calls.dispatch(Origin::signed(1)),
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(
						<Test as Config>::WeightInfo::batch_all(2) + info.weight * 2
					),
					pays_fee: Pays::Yes
				},
				error: pallet_balances::Error::<Test, _>::InsufficientBalance.into()
			}
		);
		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 10);
	});
}

fn prepare_good_block() -> (TestClient, Hash, u64, PeerId, IncomingBlock<Block>) {
	let mut client = substrate_test_runtime_client::new();
	let block = client.new_block(Default::default()).unwrap().build().unwrap().block;
	block_on(client.import(BlockOrigin::File, block)).unwrap();

	let (hash, number) = (client.block_hash(1).unwrap().unwrap(), 1);
	let header = client.header(&BlockId::Number(1)).unwrap();
	let justifications = client.justifications(&BlockId::Number(1)).unwrap();
	let peer_id = PeerId::random();
	(
		client,
		hash,
		number,
		peer_id,
		IncomingBlock {
			hash,
			header,
			body: Some(Vec::new()),
			indexed_body: None,
			justifications,
			origin: Some(peer_id),
			allow_missing_state: false,
			import_existing: false,
			state: None,
			skip_execution: false,
		},
	)
}

#[test]
fn import_single_good_block_works() {
	let (_, _hash, number, peer_id, block) = prepare_good_block();

	let mut expected_aux = ImportedAux::default();
	expected_aux.is_new_best = true;

	match block_on(import_single_block(
		&mut substrate_test_runtime_client::new(),
		BlockOrigin::File,
		block,
		//&mut PassThroughVerifier::new(true),
	)) {
		Ok(BlockImportStatus::ImportedUnknown(ref num, ref aux, ref org))
			if *num == number && *aux == expected_aux && *org == Some(peer_id) => {},
		r @ _ => panic!("{:?}", r),
	}
}