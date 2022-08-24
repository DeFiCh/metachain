
use super::*;
use substrate_test_runtime_client::{
    AccountKeyring::*, DefaultTestClientBuilderExt, TestClientBuilder, TestClientBuilderExt,
};

#[test]
fn simpleTest1() {
    let builder = TestClientBuilder::new();
    let (client, select_chain) = builder.build_with_longest_chain();
    let client = Arc::new(client);

    MetaConsensusRpc::new(client).into_rpc();
}