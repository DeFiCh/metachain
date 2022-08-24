
use super::*;
use substrate_test_runtime_client::{
    AccountKeyring::*, DefaultTestClientBuilderExt, TestClientBuilder, TestClientBuilderExt,
};
use sp_consensus::{SelectChain};

#[tokio::test]
async fn get_block_hash() {
    let builder = TestClientBuilder::new();
    let (client, select_chain) = builder.build_with_longest_chain();
    let client = Arc::new(client);

    // check if genesis block hash is returned correctly.
    assert_eq!(MetaConsensusRpc::new(client.clone()).get_block_hash(0.into()).unwrap().to_string(), select_chain.leaves().await.unwrap()[0].to_string());
}