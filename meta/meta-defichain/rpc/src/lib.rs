use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::{marker::PhantomData, sync::Arc};

pub use meta_defichain_rpc_runtime_api::DefichainApi as DefichainRuntimeApi;

const RUNTIME_ERROR: i32 = 1;

#[rpc(client, server)]
pub trait DefichainApi<BlockHash> {
	#[method(name = "defichain_get7")]
	fn get_7(&self, at: Option<BlockHash>) -> RpcResult<u64>;
}

/// A struct that implements the `DefichainApi`
pub struct Defichain<Client, Block> {
	client: Arc<Client>,
	_marker: PhantomData<Block>,
}

impl<Client, Block> Defichain<Client, Block> {
	/// Create new `Defichain` instance with the given reference to the client
	pub fn new(client: Arc<Client>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

#[async_trait]
impl<Client, Block> DefichainApiServer<<Block as BlockT>::Hash> for Defichain<Client, Block>
where
	Block: BlockT,
	Client: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	Client::Api: DefichainRuntimeApi<Block>,
{
	fn get_7(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u64> {
		let api = self.client.runtime_api();
		// to calling the runtime at specific block
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));
		api.get_7(&at).map_err(runtime_error_into_rpc_err)
	}
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(
		RUNTIME_ERROR,
		"Runtime error",
		Some(format!("{:?}", err)),
	))
	.into()
}
