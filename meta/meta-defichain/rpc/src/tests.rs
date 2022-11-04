use super::*;
use std::sync::Arc;

#[tokio::test]
async fn should_get_7() {
	let client = Arc::new(substrate_test_runtime_client::new());
	let defichain = Defichain { client: client.clone(), _marker: Default::default() };
	defichain.into_rpc();
}
