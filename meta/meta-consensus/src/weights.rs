#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_utility.
pub trait WeightInfo {
	fn batch(c: u32, ) -> Weight;
	fn as_derivative() -> Weight;
	fn batch_all(c: u32, ) -> Weight;
	fn dispatch_as() -> Weight;
	fn force_batch(c: u32, ) -> Weight;
}

/// Weights for pallet_utility using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// The range of component `c` is `[0, 1000]`.
	fn batch(c: u32, ) -> Weight {
		(23_113_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((2_701_000 as Weight).saturating_mul(c as Weight))
	}
	fn as_derivative() -> Weight {
		(4_182_000 as Weight)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn batch_all(c: u32, ) -> Weight {
		(18_682_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((2_794_000 as Weight).saturating_mul(c as Weight))
	}
	fn dispatch_as() -> Weight {
		(12_049_000 as Weight)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn force_batch(c: u32, ) -> Weight {
		(19_136_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((2_697_000 as Weight).saturating_mul(c as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// The range of component `c` is `[0, 1000]`.
	fn batch(c: u32, ) -> Weight {
		(23_113_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((2_701_000 as Weight).saturating_mul(c as Weight))
	}
	fn as_derivative() -> Weight {
		(4_182_000 as Weight)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn batch_all(c: u32, ) -> Weight {
		(18_682_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((2_794_000 as Weight).saturating_mul(c as Weight))
	}
	fn dispatch_as() -> Weight {
		(12_049_000 as Weight)
	}
	/// The range of component `c` is `[0, 1000]`.
	fn force_batch(c: u32, ) -> Weight {
		(19_136_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((2_697_000 as Weight).saturating_mul(c as Weight))
	}
}