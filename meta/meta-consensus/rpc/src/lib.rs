#![feature(assert_matches)]
//! RPC interface for the meta-concensus module
#![allow(unused_imports)] // for now
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, generic::Block as BlockS, generic::Header, traits::{Block as BlockT, Zero, NumberFor}, generic::SignedBlock};
use std::sync::Arc;
use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};

#[cfg(test)]
mod tests;

#[rpc(client, server)]
pub trait MetaConsensusRpcApi<Block>
where 
    Block: BlockT
{
	#[method(name = "metaConsensusRpc_getBlockHash")]
	fn get_block_hash(&self, at: Option<NumberFor<Block>> ) -> RpcResult<String>;

    #[method(name = "metaConsensusRpc_getBlock")]
	fn get_block(&self, at: Option<NumberFor<Block>> ) -> RpcResult<Vec<u8>>;
}

/// A struct that implements the `MetaConsensusRpcApiServer`.
pub struct MetaConsensusRpc<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> MetaConsensusRpc<C, M> {
	/// Create new `MetaConsensusRpcApi` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

#[async_trait]
impl<C, Block> MetaConsensusRpcApiServer<Block> for MetaConsensusRpc<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	// C::Api: MetaConsensusRpcRuntimeApi<Block>,
    C: sc_client_api::BlockBackend<Block>,
{
	fn get_block_hash(&self, at: Option<NumberFor<Block>>) -> RpcResult<String> {
		let block_num = at.unwrap_or_else(|| {
			// If the block number is not supplied assume the best block.
			self.client.info().best_number
        });
        let block_hash = self.client.block_hash(block_num.into()).unwrap().unwrap_or_default();
        
		Ok(block_hash.to_string())
	}

    fn get_block(&self, at: Option<NumberFor<Block>>) -> RpcResult<Vec<u8>> {
		let block_num = at.unwrap_or_else(|| {
			// If the block number is not supplied assume the best block.
			self.client.info().best_number
        });
        let block = self.client.block(&BlockId::Number(block_num)).unwrap().unwrap(); // NOTE(surangap): unwrap_or_default

		let encoded = BlockS::encode_from(block.block.header(), block.block.extrinsics());
        
		Ok(encoded)
	}
}
