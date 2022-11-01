//! The Substrate runtime. This can be compiled with `#[no_std]`, ready for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::AtLeast32BitUnsigned;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		// The type used to store balances.
		type Balance: MaxEncodedLen + Member + Parameter + AtLeast32BitUnsigned + Default + Copy;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Storage item for balances to accounts mapping.
	#[pallet::storage]
	#[pallet::getter(fn get_balance)]
	pub(super) type BalanceToAccount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	/// Token mint can emit two Event types.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New token supply was minted.
		MintedNewSupply(T::AccountId),
		/// Tokens were successfully transferred between accounts. [from, to, value]
		Transferred(T::AccountId, T::AccountId, T::Balance),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	pub fn get_7() -> u64 {
		7
	}
}
