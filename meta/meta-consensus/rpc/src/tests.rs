use std::borrow::{Borrow, BorrowMut};
use super::*;
use futures::prelude::*;
use futures::executor::block_on;
use sc_basic_authorship::ProposerFactory;
use sc_client_api::BlockBackend;
use sc_consensus::{
    ImportedAux,
    block_import::{BlockImport, BlockImportParams, ForkChoiceStrategy, ImportResult},
    import_queue::{BasicQueue, BoxBlockImport, Verifier},
};
use sc_transaction_pool::{BasicPool, Options, RevalidationType};
use sc_transaction_pool_api::{MaintainedTransactionPool, TransactionPool, TransactionSource};
use sp_inherents::InherentData;
use sp_consensus::{SelectChain, BlockOrigin};
use codec::{Codec, Decode, Encode};
use futures::channel::oneshot::{Receiver, Sender};
use futures::channel::mpsc::{Sender as MPSCSender};
use sp_runtime::{generic::{Block, Header, SignedBlock, BlockId, Digest, DigestItem}, print, traits::{Block as BlockT, }};
use substrate_test_runtime_client::{
    AccountKeyring::*, DefaultTestClientBuilderExt, TestClientBuilder, TestClientBuilderExt,
};
use substrate_test_runtime_transaction_pool::{uxt, TestApi};
use meta_runtime::{
    Header as MetaHeader, opaque::UncheckedExtrinsic as MetaUncheckedExtrinsic, Hash,
    opaque::Block as MetaBlock,
};
use sc_consensus_manual_seal::{
    ConsensusDataProvider, ManualSealParams,
    Error,
    rpc::{CreatedBlock, EngineCommand},
    seal_block, SealBlockParams, MAX_PROPOSAL_DURATION,
    finalize_block, FinalizeBlockParams,
};
use sp_api::{ProvideRuntimeApi, TransactionFor};
use sp_blockchain::HeaderBackend;
use sp_core::{OpaqueMetadata, H160, H256, U256};
use substrate_test_runtime::{ Extrinsic as TestExtrinsic,  Block as TestBlock };
use sc_block_builder::BlockBuilderProvider;


fn api() -> Arc<TestApi> {
    Arc::new(TestApi::empty())
}

const SOURCE: TransactionSource = TransactionSource::External;

struct TestDigestProvider<C> {
    _client: Arc<C>,
}

impl<B, C> ConsensusDataProvider<B> for TestDigestProvider<C>
    where
        B: BlockT,
        C: ProvideRuntimeApi<B> + Send + Sync,
{
    type Transaction = TransactionFor<C, B>;

    fn create_digest(
        &self,
        _parent: &B::Header,
        _inherents: &InherentData,
    ) -> Result<Digest, Error> {
        Ok(Digest { logs: vec![] })
    }

    fn append_block_import(
        &self,
        _parent: &B::Header,
        params: &mut BlockImportParams<B, Self::Transaction>,
        _inherents: &InherentData,
    ) -> Result<(), Error> {
        params.post_digests.push(DigestItem::Other(vec![1]));
        Ok(())
    }
}

async fn mint_block(sink: &mut MPSCSender<EngineCommand<Hash>>)
{
    let (tx, rx) = futures::channel::oneshot::channel();
    sink.send(EngineCommand::SealNewBlock {
        parent_hash: None,
        sender: Some(tx),
        create_empty: true,
        finalize: true,
    })
        .await
        .unwrap();
    let created_block = rx.await.unwrap().unwrap();
    assert_eq!(
        created_block,
        CreatedBlock {
            hash: created_block.hash.clone(),
            aux: ImportedAux {
                header_only: false,
                clear_justification_requests: false,
                needs_justification: false,
                bad_justification: false,
                is_new_best: true,
            }
        }
    );
}

#[tokio::test]
async fn get_block_hash() {
    let builder = TestClientBuilder::new();
    let (client, select_chain) = builder.build_with_longest_chain();
    let client = Arc::new(client);
    let meta_consensus_rpc = MetaConsensusRpc::new(client.clone(), None, client.clone());

    // check for the best block(0 = genesis)
    let best_hash_check = || {
        assert_eq!(meta_consensus_rpc.get_block_hash(client.info().best_number.into()).unwrap().to_string(), client.info().best_hash.to_string());
    };
    assert_eq!(client.info().best_number, 0);
    best_hash_check();

    let pool_api = api();
    let spawner = sp_core::testing::TaskExecutor::new();
    let pool = Arc::new(BasicPool::with_revalidation_type(
        Options::default(),
        true.into(),
        pool_api.clone(),
        None,
        RevalidationType::Full,
        spawner.clone(),
        0,
    ));
    let env = ProposerFactory::new(spawner.clone(), client.clone(), pool.clone(), None, None);

    // create mpsc sender and receiver
    let (mut sink, commands_stream) = futures::channel::mpsc::channel(1024);
    let future = sc_consensus_manual_seal::run_manual_seal(ManualSealParams {
        block_import: client.clone(),
        env,
        client: client.clone(),
        pool: pool.clone(),
        commands_stream,
        select_chain: select_chain.clone(),
        consensus_data_provider: None,
        create_inherent_data_providers: |_, _| async { Ok(()) },
    });
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // spawn the background authorship task
        rt.block_on(future);
    });

    // mint a block now
    mint_block(sink.borrow_mut()).await;
    // check for the best block(1)
    assert_eq!(client.info().best_number, 1);
    best_hash_check();

    // Add a transaction and mint another block
    // submit a transaction to pool.
    let result = pool.submit_one(&BlockId::Number(0), SOURCE, uxt(Alice, 0)).await;
    // assert that it was successfully imported
    assert!(result.is_ok());
    mint_block(sink.borrow_mut()).await;

    // check for the best block(2)
    assert_eq!(client.info().best_number, 2);
    best_hash_check();
}

#[tokio::test]
async fn header_encode_decode() {
    let builder = TestClientBuilder::new();
    let (client, _select_chain) = builder.build_with_longest_chain();
    let client = Arc::new(client);

    let genesis_block = client.block(&BlockId::Number(0)).unwrap().unwrap();
    let encoded_header = genesis_block.block.header().encode();
    let decoded_header = MetaHeader::decode(&mut &encoded_header[..]).unwrap();


    // check if we get the same results
    assert_eq!(genesis_block.block.header().hash(), decoded_header.hash());
    assert_eq!(genesis_block.block.header().parent_hash, decoded_header.parent_hash);
    assert_eq!(genesis_block.block.header().number, decoded_header.number as u64);
    assert_eq!(genesis_block.block.header().state_root, decoded_header.state_root);
    assert_eq!(genesis_block.block.header().extrinsics_root, decoded_header.extrinsics_root);
    assert_eq!(genesis_block.block.header().digest, decoded_header.digest);

}

#[tokio::test]
async fn extrinsics_encode_decode() {
    let builder = TestClientBuilder::new();
    let (client, select_chain) = builder.build_with_longest_chain();
    let client = Arc::new(client);
    let pool_api = api();
    let spawner = sp_core::testing::TaskExecutor::new();
    let pool = Arc::new(BasicPool::with_revalidation_type(
        Options::default(),
        true.into(),
        pool_api.clone(),
        None,
        RevalidationType::Full,
        spawner.clone(),
        0,
    ));
    let env = ProposerFactory::new(spawner.clone(), client.clone(), pool.clone(), None, None);

    // create mpsc sender and receiver
    let (mut sink, commands_stream) = futures::channel::mpsc::channel(1024);
    let future = sc_consensus_manual_seal::run_manual_seal(ManualSealParams {
        block_import: client.clone(),
        env,
        client: client.clone(),
        pool: pool.clone(),
        commands_stream,
        select_chain: select_chain.clone(),
        consensus_data_provider: None,
        create_inherent_data_providers: |_, _| async { Ok(()) },
    });
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // spawn the background authorship task
        rt.block_on(future);
    });

    // Add a transaction and mint another block so that we have an extrinsic
    // submit a transaction to pool.
    let result = pool.submit_one(&BlockId::Number(0), SOURCE, uxt(Alice, 0)).await;
    // assert that it was successfully imported
    assert!(result.is_ok());
    mint_block(sink.borrow_mut()).await;

    let best_block = client.block(&BlockId::Number(client.info().best_number)).unwrap().unwrap();
    if best_block.block.extrinsics().len() > 0 {
        let encoded_extrinsic = best_block.block.extrinsics()[0].encode();
        let decoded_extrinsic = TestExtrinsic::decode(&mut &encoded_extrinsic[..]).unwrap();

        assert_eq!(best_block.block.extrinsics()[0].transfer().from.to_string(), Alice.to_account_id().to_string());
        assert_eq!(best_block.block.extrinsics()[0].transfer().from, decoded_extrinsic.transfer().from);
        assert_eq!(best_block.block.extrinsics()[0].transfer().to, decoded_extrinsic.transfer().to);
        assert_eq!(best_block.block.extrinsics()[0].transfer().amount, decoded_extrinsic.transfer().amount);
        // TODO(surangap): deconstruct, extract and match the signature as well.
    }
}

#[tokio::test]
async fn block_encode_decode() {
    let builder = TestClientBuilder::new();
    let (client, select_chain) = builder.build_with_longest_chain();
    let client = Arc::new(client);
    let pool_api = api();
    let spawner = sp_core::testing::TaskExecutor::new();
    let pool = Arc::new(BasicPool::with_revalidation_type(
        Options::default(),
        true.into(),
        pool_api.clone(),
        None,
        RevalidationType::Full,
        spawner.clone(),
        0,
    ));
    let env = ProposerFactory::new(spawner.clone(), client.clone(), pool.clone(), None, None);

    // create mpsc sender and receiver
    let (mut sink, commands_stream) = futures::channel::mpsc::channel(1024);
    let future = sc_consensus_manual_seal::run_manual_seal(ManualSealParams {
        block_import: client.clone(),
        env,
        client: client.clone(),
        pool: pool.clone(),
        commands_stream,
        select_chain: select_chain.clone(),
        consensus_data_provider: None,
        create_inherent_data_providers: |_, _| async { Ok(()) },
    });
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // spawn the background authorship task
        rt.block_on(future);
    });

    // Add transactions and mint another block so that we have an extrinsic
    // submit two transactions to pool.
    let result = pool.submit_at(&BlockId::Number(0), SOURCE, vec![uxt(Alice, 0), uxt(Alice, 1)]).await;
    // assert that it was successfully imported
    assert!(result.is_ok());

    mint_block(sink.borrow_mut()).await;

    let best_block = client.block(&BlockId::Number(client.info().best_number)).unwrap().unwrap();
    if best_block.block.extrinsics().len() > 0 {
        let encoded_block = best_block.block.encode();
        let decoded_block = TestBlock::decode(&mut &encoded_block[..]).unwrap();
        assert!(best_block.block == decoded_block);
    }

    // check SignedBlock en-dec
    let best_signed_block = client.block(&BlockId::Number(client.info().best_number)).unwrap().unwrap();
    if best_signed_block.block.extrinsics().len() > 0 {
        let encoded_block = best_signed_block.encode();
        let decoded_signed_block = SignedBlock::decode(&mut &encoded_block[..]).unwrap();
        assert!(best_signed_block == decoded_signed_block);
    }
}

#[tokio::test]
async fn block_import() {
    let builder = TestClientBuilder::new();
    let (client, select_chain) = builder.build_with_longest_chain();
    let mut client = Arc::new(client);
    let pool_api = api();
    let spawner = sp_core::testing::TaskExecutor::new();
    let pool = Arc::new(BasicPool::with_revalidation_type(
        Options::default(),
        true.into(),
        pool_api.clone(),
        None,
        RevalidationType::Full,
        spawner.clone(),
        0,
    ));
    let env = ProposerFactory::new(spawner.clone(), client.clone(), pool.clone(), None, None);

    // create mpsc sender and receiver
    let (mut sink, commands_stream) = futures::channel::mpsc::channel(1024);
    let future = sc_consensus_manual_seal::run_manual_seal(ManualSealParams {
        block_import: client.clone(),
        env,
        client: client.clone(),
        pool: pool.clone(),
        commands_stream,
        select_chain: select_chain.clone(),
        consensus_data_provider: None,
        create_inherent_data_providers: |_, _| async { Ok(()) },
    });
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // spawn the background authorship task
        rt.block_on(future);
    });

    // Add transactions and mint another block so that we have an extrinsic
    // submit two transactions to pool.
    let result = pool.submit_at(&BlockId::Number(0), SOURCE, vec![uxt(Alice, 0), uxt(Alice, 1)]).await;
    // assert that it was successfully imported
    assert!(result.is_ok());

    mint_block(sink.borrow_mut()).await;

    let best_block = client.block(&BlockId::Number(client.info().best_number)).unwrap().unwrap();
    assert!(best_block.block.extrinsics().len() > 0);
    let encoded_block = best_block.block.encode();
    let decoded_block = TestBlock::decode(&mut &encoded_block[..]).unwrap();
    assert!(best_block.block == decoded_block);
    let best_block_before_import = client.info().best_number;

    {
        // try to import the previous block. should get an error
        let (header, extrinsics) = decoded_block.deconstruct();
        let mut import = BlockImportParams::new(BlockOrigin::Own, header);
        import.body = Some(extrinsics);
        import.fork_choice = Some(ForkChoiceStrategy::LongestChain);
        let import_result = client.import_block(import, Default::default()).await;
        assert_eq!(import_result.unwrap(), ImportResult::AlreadyInChain);
    }

    {
        // build a new block and try to import.
        let new_block = client.new_block(Default::default()).unwrap().build().unwrap().block;
        let mut import = BlockImportParams::new(BlockOrigin::Own, new_block.header);
        import.body = Some(new_block.extrinsics);
        import.fork_choice = Some(ForkChoiceStrategy::LongestChain);
        import.finalized = true;

        let import_result = client.import_block( import, Default::default()).await;
        assert_eq!(import_result.unwrap(), ImportResult::Imported(ImportedAux { is_new_best: true, ..Default::default() }));
        let best_block_after_import = client.info().best_number;
        assert_eq!(best_block_after_import, best_block_before_import + 1);
    }
}

#[tokio::test]
async fn block_mint() {
    let builder = TestClientBuilder::new();
    let (client, select_chain) = builder.build_with_longest_chain();
    let client = Arc::new(client); // TODO(surangap): change this to use meta client
    let pool_api = api();
    let spawner = sp_core::testing::TaskExecutor::new();
    let pool = Arc::new(BasicPool::with_revalidation_type(
        Options::default(),
        true.into(),
        pool_api.clone(),
        None,
        RevalidationType::Full,
        spawner.clone(),
        0,
    ));
    let env = ProposerFactory::new(spawner.clone(), client.clone(), pool.clone(), None, None);

    // create mpsc sender and receiver
    let ( sink, commands_stream) = futures::channel::mpsc::channel(1024);
    let future = sc_consensus_manual_seal::run_manual_seal(ManualSealParams {
        block_import: client.clone(),
        env,
        client: client.clone(),
        pool: pool.clone(),
        commands_stream,
        select_chain: select_chain.clone(),
        consensus_data_provider: None,
        create_inherent_data_providers: |_, _| async { Ok(()) },
    });
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // spawn the background authorship task
        rt.block_on(future);
    });

    // Add transactions and mint another block so that we have an extrinsic
    // submit two transactions to pool.
    let result = pool.submit_at(&BlockId::Number(0), SOURCE, vec![uxt(Alice, 0), uxt(Alice, 1)]).await;
    // assert that it was successfully imported
    assert!(result.is_ok());
    let block_height_before = client.info().best_number;
    let block_hash_before = client.info().best_hash;

    // mint block using meta-consensus-rpc
    let meta_consensus_rpc = MetaConsensusRpc::new(client.clone(), Some(sink), client.clone());
    let (dmc_payload, dmc_txs) = meta_consensus_rpc.mint_block(Default::default()).await.unwrap();

    // decode and check results
    assert_eq!(dmc_txs.len(), 0); // for now
    let decoded_signed_block: SignedBlock<TestBlock> = SignedBlock::decode(&mut &dmc_payload[..]).unwrap();
    assert_eq!(decoded_signed_block.block.header().number as u64, block_height_before + 1);
    assert_eq!(decoded_signed_block.block.header().parent_hash, block_hash_before);
}

#[tokio::test]
async fn block_import_meta_consensus_rpc() {
    let builder = TestClientBuilder::new();
    let (client, select_chain) = builder.build_with_longest_chain();
    let client = Arc::new(client);
    let pool_api = api();
    let spawner = sp_core::testing::TaskExecutor::new();
    let pool = Arc::new(BasicPool::with_revalidation_type(
        Options::default(),
        true.into(),
        pool_api.clone(),
        None,
        RevalidationType::Full,
        spawner.clone(),
        0,
    ));
    let env = ProposerFactory::new(spawner.clone(), client.clone(), pool.clone(), None, None);

    // create mpsc sender and receiver
    let (mut sink, commands_stream) = futures::channel::mpsc::channel(1024);
    let future = sc_consensus_manual_seal::run_manual_seal(ManualSealParams {
        block_import: client.clone(),
        env,
        client: client.clone(),
        pool: pool.clone(),
        commands_stream,
        select_chain: select_chain.clone(),
        consensus_data_provider: None,
        create_inherent_data_providers: |_, _| async { Ok(()) },
    });
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // spawn the background authorship task
        rt.block_on(future);
    });

    // Add transactions and mint another block so that we have an extrinsic
    // submit two transactions to pool.
    let result = pool.submit_at(&BlockId::Number(0), SOURCE, vec![uxt(Alice, 0), uxt(Alice, 1)]).await;
    // assert that it was successfully imported
    assert!(result.is_ok());

    mint_block(sink.borrow_mut()).await;

    let best_block = client.block(&BlockId::Number(client.info().best_number)).unwrap().unwrap();
    assert!(best_block.block.extrinsics().len() > 0);
    let encoded_block = best_block.encode();
    let best_block_before_import = client.info().best_number;

    // import the blocks using meta-consensus-rpc
    let meta_consensus_rpc = MetaConsensusRpc::new(client.clone(), Some(sink), client.clone());

    {
        // try to import the previous block. should not succeed.
        let (import_result, dmc_txs) = meta_consensus_rpc.connect_block( encoded_block, Default::default()).await.unwrap();
        assert!(!import_result);
    }
    {
        // build a new block and try to import.
        let new_block = client.new_block(Default::default()).unwrap().build().unwrap().block;
        let new_signed_block = SignedBlock{block: new_block.clone(), justifications:None};

        let (import_result, dmc_txs) = meta_consensus_rpc.connect_block( new_signed_block.encode(), Default::default()).await.unwrap();
        assert!(import_result);
        assert_eq!(client.info().best_number, best_block_before_import + 1);
        assert_eq!(client.info().best_hash, new_block.header.hash());
    }
}





