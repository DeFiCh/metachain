#![warn(missing_docs)]

use std::sync::Arc;

use futures::channel::mpsc::Sender;
use meta_runtime::{opaque::Block, Hash,};
use sc_consensus_manual_seal::{
	rpc::{ManualSeal, ManualSealApi},
	EngineCommand,
};
pub use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sc_transaction_pool_api::TransactionPool;

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
pub fn create_full<C, P>(deps: FullDeps<C, P>) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
{
	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps {
		command_sink,
		client,
		..
	} = deps;

	// The RPC extension receives commands for the manual seal consensus engine.
	io.extend_with(
		// We provide the rpc handler with the sending end of the channel to allow the rpc
		// send EngineCommands to the background block authorship task.
		ManualSealApi::to_delegate(ManualSeal::new(command_sink)),
	);

	io
}
