//! RPC interface for the meta-concensus module
#![allow(unused_imports)]

use std::borrow::BorrowMut;
use std::ops::{Add, Deref};
// for now
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::{Block as BlockS, Header, SignedBlock, BlockId, Digest, DigestItem}, print, traits::{Block as BlockT, Zero, NumberFor  }};
use std::sync::Arc;
use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};
use codec::{Codec, Decode, Encode};
use futures::channel::oneshot::*;
use futures::channel::mpsc::{Sender as MPSCSender};
use futures::prelude::*;
use sc_consensus_manual_seal::{
	ConsensusDataProvider, ManualSealParams,
	Error,
	rpc::{CreatedBlock, EngineCommand},
	seal_block, SealBlockParams, MAX_PROPOSAL_DURATION,
	finalize_block, FinalizeBlockParams,
};
use meta_runtime::{Header as MetaHeader, opaque::UncheckedExtrinsic as MetaUncheckedExtrinsic, Hash, BlockNumber };
use sp_core::{OpaqueMetadata, H160, H256, U256};
use sc_consensus::{
	ImportedAux,
	block_import::{BlockImport, BlockImportParams, ForkChoiceStrategy, ImportResult},
	import_queue::{BasicQueue, BoxBlockImport, Verifier},
};
use sp_consensus::{SelectChain, BlockOrigin};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DNCTx {
	from: String,
	to: String,
	amount: i64,
	signature: String
}

// NOTE(surangap): keeping DMCTx as separate struct from DNCTx for now
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DMCTx {
	from: String,
	to: String,
	amount: i64,
	signature: String
}

#[rpc(client, server)]
pub trait MetaConsensusRpcApi<Block>
where 
    Block: BlockT
{
	#[method(name = "metaConsensusRpc_getBlockHash")]
	fn get_block_hash(&self, at: Option<NumberFor<Block>> ) -> RpcResult<String>;

    #[method(name = "metaConsensusRpc_getBlock")]
	fn get_block(&self, at: Option<NumberFor<Block>> ) -> RpcResult<Vec<u8>>;

	#[method(name = "metaConsensusRpc_mintBlock")]
	async fn mint_block(&self, dnc_txs: Vec<DNCTx> ) -> RpcResult<(Vec<u8>, Vec<DMCTx>)>;

	#[method(name = "metaConsensusRpc_connectBlock")]
	async fn connect_block(&self, dmc_payload: Vec<u8>, dnc_txs: Vec<DNCTx> ) -> RpcResult<(bool, Vec<DMCTx>)>;
}

/// A struct that implements the `MetaConsensusRpcApiServer`.
pub struct MetaConsensusRpc<C, M, I> {
	client: Arc<C>,
	command_sink: Option<MPSCSender<EngineCommand<Hash>>>,
	block_import: I,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M, I> MetaConsensusRpc<C, M, I> {
	/// Create new `MetaConsensusRpc` instance
	pub fn new(client: Arc<C>, command_sink: Option<MPSCSender<EngineCommand<Hash>>>, block_import: I ) -> Self {
		Self {
			client,
			command_sink,
			block_import,
			_marker: Default::default(),
		}
	}
}

#[async_trait]
impl<C, Block, I> MetaConsensusRpcApiServer<Block> for MetaConsensusRpc<C, Block, I>
where
	Block: BlockT<Hash = Hash>,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	// C::Api: MetaConsensusRpcRuntimeApi<Block>,
    C: sc_client_api::BlockBackend<Block>,
	C: BlockImport<Block>,
	I: BlockImport<Block, Transaction = sp_api::TransactionFor<C, Block>> + Send + Sync + Clone + 'static,
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

		match self.client.block(&BlockId::Number(block_num)).unwrap() {
			Some(signed_block) => Ok(signed_block.encode()),
			_ => Err(Error::StringError(format!("Requested block number [{}] does not exist.", block_num)).into()), // TODO(surangap): Define errors.
		}
	}

	async fn mint_block(&self, dnc_txs: Vec<DNCTx> ) -> RpcResult<(Vec<u8>, Vec<DMCTx>)> {
		//TODO(surangap): validate the dnc_txs. do the account balance changes accordingly

		// send command to mint the next block
		let sink = self.command_sink.clone();
		let (sender, receiver) = futures::channel::oneshot::channel();
		let parent_hash = self.client.info().best_hash;
		let command = EngineCommand::SealNewBlock {
			create_empty: true,
			finalize: true,
			parent_hash: Some(parent_hash),
			sender: Some(sender),
		};
		sink.unwrap().send(command).await?;

		match receiver.await {
			Ok(Ok(rx)) => {
				assert_eq!(rx.hash, self.client.info().best_hash);
				let new_block = self.client.block(&BlockId::Number(self.client.info().best_number));
				// extract DMCTxs to send
				let dmc_txs: Vec<DMCTx> = Default::default(); // TODO(surangap): extract the DMCTx based on the relevant criteria
				match new_block.unwrap() {
					Some(signed_block) => Ok((signed_block.encode(), dmc_txs)),
					_ => Err(Error::StringError("Block minting error.".to_string()).into()), // TODO(surangap): Define errors.
				}
			},
			Ok(Err(e)) => Err(e.into()),
			Err(e) => Err(JsonRpseeError::to_call_error(e)),
		}
	}

	async fn connect_block(&self, dmc_payload: Vec<u8>, dnc_txs: Vec<DNCTx> ) -> RpcResult<(bool, Vec<DMCTx>)> {
		// 	decode the signed block
		let decoded = SignedBlock::decode(&mut &dmc_payload[..]);
		if let Err(e) = decoded {
			//return with proper error
			return Err(JsonRpseeError::to_call_error(e))
		}
		let signed_block: SignedBlock<Block>  = decoded.unwrap();

		// import the block.
		let (header, extrinsics) = signed_block.block.deconstruct();
		let mut import = BlockImportParams::new(BlockOrigin::NetworkBroadcast, header);
		import.body = Some(extrinsics);
		import.fork_choice = Some(ForkChoiceStrategy::LongestChain);
		import.finalized = true;
		let mut block_import = self.block_import.clone(); // NOTE(surangap): check if this is semantically correct.
		let import_result = block_import.import_block(import, Default::default()).await;

		match import_result {
			Ok( ImportResult::Imported(aux)) => {
				let dmc_txs: Vec<DMCTx> = Default::default() ; // TODO(surangapa): extract the DMCTxs in the block that just got imported.
				Ok((true, dmc_txs))
			},
			_ => Err(Error::StringError("Block importing error.".to_string()).into()), // TODO(surangap): more descriptive error
		}
	}
}
