
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

    // check for the first block hash
    if select_chain.leaves().await.unwrap().len() > 1 {
        assert_eq!(MetaConsensusRpc::new(client.clone()).get_block_hash(1.into()).unwrap().to_string(), select_chain.leaves().await.unwrap()[1].to_string());
    }

    // should return best block hash when None block number is given
    let block_height = select_chain.leaves().await.unwrap().len();
    assert_eq!(MetaConsensusRpc::new(client.clone()).get_block_hash(None).unwrap().to_string(), select_chain.leaves().await.unwrap()[block_height - 1].to_string());
}

