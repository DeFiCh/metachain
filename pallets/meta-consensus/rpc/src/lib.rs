//! RPC interface for the transaction payment module.

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, generic::Block as BlockS, generic::Header as HeaderS, traits::{Block as BlockT, Zero, NumberFor}};
use std::sync::Arc;
use meta_consensus_runtime_api::MetaConsensusApi as MetaConsensusRuntimeApi;

#[rpc]
pub trait MetaConsensusApi<Block>
where 
    Block: BlockT
{
	#[rpc(name = "metaConsensus_getBlockHash")]
	fn get_block_hash(&self, at: Option<NumberFor<Block>> ) -> Result<String>;

    #[rpc(name = "metaConsensus_getBlock")]
	fn get_block(&self, at: Option<NumberFor<Block>> ) -> Result<Vec<u8>>;
}

/// A struct that implements the `MetaConsensusApi`.
pub struct MetaConsensus<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> MetaConsensus<C, M> {
	/// Create new `MetaConsensus` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

/// Error type of this RPC api.
// pub enum Error {
// 	/// The transaction was not decodable.
// 	DecodeError,
// 	/// The call to runtime failed.
// 	RuntimeError,
// }
//
// impl From<Error> for i64 {
// 	fn from(e: Error) -> i64 {
// 		match e {
// 			Error::RuntimeError => 1,
// 			Error::DecodeError => 2,
// 		}
// 	}
// }

impl<C, Block> MetaConsensusApi<Block> for MetaConsensus<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	// C::Api: MetaConsensusRuntimeApi<Block>,
	// C::Api: sc_rpc_api::chain::ChainApi<Number, Hash, Header, SignedBlock>,
    // C: sc_rpc::chain::ChainBackend<C, Block>
    C: sc_client_api::BlockBackend<Block>
{
	fn get_block_hash(&self, at: Option<NumberFor<Block>>) -> Result<String> {
		let block_num = at.unwrap_or_else(|| {
			// If the block number is not supplied assume the best block.
			self.client.info().best_number
        });
        let block_hash = self.client.block_hash(block_num.into()).unwrap().unwrap_or_default();
        
		Ok(block_hash.to_string())
	}

	// NOTE(surangap): WIP, need to check encode of the SginedBlock and then decode,deconstruct the block
    fn get_block(&self, at: Option<NumberFor<Block>>) -> Result<Vec<u8>> {
		let block_num = at.unwrap_or_else(|| {
			// If the block number is not supplied assume the best block.
			self.client.info().best_number
        });
        let block = self.client.block(&BlockId::Number(block_num)).unwrap().unwrap(); // NOTE(surangap): unwrap_or_default

		let encoded = BlockS::encode_from(block.block.header(), block.block.extrinsics());
        
		Ok(encoded)
	}
}
