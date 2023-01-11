use super::*;
use jsonrpsee::types::EmptyParams;
use sp_api::{ApiRef, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, NumberFor, Zero},
};
use std::sync::Arc;
use substrate_test_runtime_client::runtime::Block;

pub struct TestApi {}

pub struct TestRuntimeApi {}

impl ProvideRuntimeApi<Block> for TestApi {
	type Api = TestRuntimeApi;

	fn runtime_api<'a>(&'a self) -> ApiRef<'a, Self::Api> {
		TestRuntimeApi {}.into()
	}
}

/// Blockchain database header backend. Does not perform any validation.
impl<Block: BlockT> HeaderBackend<Block> for TestApi {
	fn header(
		&self,
		_id: BlockId<Block>,
	) -> std::result::Result<Option<Block::Header>, sp_blockchain::Error> {
		Ok(None)
	}

	fn info(&self) -> sc_client_api::blockchain::Info<Block> {
		sc_client_api::blockchain::Info {
			best_hash: Default::default(),
			best_number: Zero::zero(),
			finalized_hash: Default::default(),
			finalized_number: Zero::zero(),
			genesis_hash: Default::default(),
			number_leaves: Default::default(),
			finalized_state: None,
			block_gap: None,
		}
	}

	fn status(
		&self,
		_id: BlockId<Block>,
	) -> std::result::Result<sc_client_api::blockchain::BlockStatus, sp_blockchain::Error> {
		Ok(sc_client_api::blockchain::BlockStatus::Unknown)
	}

	fn number(
		&self,
		_hash: Block::Hash,
	) -> std::result::Result<Option<NumberFor<Block>>, sp_blockchain::Error> {
		Ok(None)
	}

	fn hash(
		&self,
		_number: NumberFor<Block>,
	) -> std::result::Result<Option<Block::Hash>, sp_blockchain::Error> {
		Ok(None)
	}
}

sp_api::mock_impl_runtime_apis! {
	impl meta_defichain_rpc_runtime_api::DefichainApi<Block> for TestRuntimeApi {
		fn get_7() -> u64 {
			7
		}
	}
}

#[tokio::test]
async fn should_get_7() {
	let client = Arc::new(TestApi {});
	let defichain = Defichain { client: client.clone(), _marker: Default::default() };
	let api = defichain.into_rpc();
	let res = api.call::<_, u64>("defichain_get7", EmptyParams::new()).await.unwrap();
	assert_eq!(res, 7);
}
