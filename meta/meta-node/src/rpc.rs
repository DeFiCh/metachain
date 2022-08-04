#![warn(missing_docs)]

use std::sync::Arc;

use jsonrpsee::RpcModule;
use futures::channel::mpsc::Sender;
use meta_runtime::{opaque::Block, AccountId, Balance, Index, Hash};
use sc_consensus_manual_seal::{
	rpc::{ManualSeal, ManualSealApiServer},
	EngineCommand,
};
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// Full client dependencies.
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// A command stream to send authoring commands to manual seal consensus engine
	pub command_sink: Sender<EngineCommand<Hash>>,
}

// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	// use substrate_frame_rpc_system::{System, SystemApiServer};
	
	// let mut io = jsonrpc_core::IoHandler::default();
	let mut module = RpcModule::new(());
	let FullDeps { client, pool, deny_unsafe, command_sink } = deps;

	// The RPC extension receives commands for the manual seal consensus engine.
	// io.extend_with(
	// 	// We provide the rpc handler with the sending end of the channel to allow the rpc
	// 	// send EngineCommands to the background block authorship task.
	// 	ManualSealApi::to_delegate(ManualSeal::new(command_sink)),
	// );

	// module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	// Extend this RPC with a custom API by using the following syntax.
	// `YourRpcStruct` should have a reference to a client, which is needed
	// to call into the runtime.
	// `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`
	module.merge(ManualSealApiServer::into_rpc(ManualSeal::new(command_sink)))?;

	Ok(module)
}