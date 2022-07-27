use super::*;

#[test]
fn test_should_work() {
 new_test_ext().execute_with(|| {
   assert_ok!(
        Call::<Test>::get_block_hash(
        Origin::signed(1),
        Some(BlockId::Number(1))
        )
   );
 })
}

#[test]
fn test_decode_block_hash() {
    new_test_ext().execute_with(|| {
    assert_eq!(
        Call::<Test>::get_block_hash(
            Origin::signed(1),
            Some(BlockId::Number(1))
        ),
        Ok(hex!("0x0000000000000000000000000000000000000000000000000000000000000000").to_string())
    );
    })
}

fn test_insert_block_into_import_queue() {
    new_test_ext().execute_with(|| {
        let block = Block::new(
            Header {
                number: 1,
                parent_hash: [0; 32].into(),
                state_root: [0; 32].into(),
                extrinsics_root: [0; 32].into(),
                digest: Digest {
                    logs: vec![],
                },
            },
            vec![],
        );
        let encoded = BlockS::encode_from(block.header(), block.extrinsics());
        let block_hash = block.header().hash();
        let block_num = block.header().number;
        let block_id = BlockId::Number(block_num);
        let block_header = block.header();
        let block_extrinsics = block.extrinsics();
        let block_body = BlockBody {
            extrinsics: block_extrinsics,
        };
        let block_data = BlockData {
            header: block_header,
            extrinsics: block_extrinsics,
            body: block_body,
        };
        let block_info = BlockInfo {
            hash: block_hash,
            number: block_num,
            parent_hash: block_header.parent_hash,
            state_root: block_header.state_root,
            extrinsics_root: block_header.extrinsics_root,
            digest: block_header.digest,
        };
        let block_data_with_info = BlockDataWithInfo {
            data: block_data,
            info: block_info,
        };
        let block_with_info = BlockWithInfo {
            data: block_data_with_info,
            info: block_info,
        };
        let block_with_info_and_extrinsics = BlockWithInfoAndExtrinsics {
            data: block_data_with_info,
            info: block_info,
            extrinsics: block_extrinsics,
        };
        let block_with_info_and_extrinsics_and_header = BlockWithInfoAndExtrinsicsAndHeader
        {
            data: block_data_with_info,
            info: block_info,
            extrinsics: block_extrinsics,
            header: block_header,
        };
        let mut import_queue = ImportQueue::<Block>::new(
            Arc::new(DummyBlockImport::<Block>),
            Arc::new(DummyJustificationImport::<Block>),
            Arc::new(DummyFinalityProofImport::<Block>),
        );
        import_queue.import_blocks(
            vec![block_with_info_and_extrinsics],
            HashMap::new(),
            Vec::new(),
        );
        assert_eq!(
            import_queue.import_queue.lock().unwrap().queue.len(),
            1
        );
    });
}