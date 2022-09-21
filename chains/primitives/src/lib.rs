#![cfg_attr(not(feature = "std"), no_std)]
use sp_core::H256;
use sp_runtime::{
	generic,
	traits::{IdentifyAccount, Verify},
	MultiSignature,
};

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Type of block number.
pub type BlockNumber = u32;

/// Balance of an account.
pub type Balance = u128;

/// Digest item type.
pub type DigestItem = generic::DigestItem;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = H256;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;