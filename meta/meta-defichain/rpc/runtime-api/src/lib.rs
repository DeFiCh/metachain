#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
	pub trait DefichainApi {
		fn get_7() -> u64;
	}
}
