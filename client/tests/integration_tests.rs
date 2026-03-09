use avail_rust_client::{
	block::{BlockExtrinsicMetadata, UntypedExtrinsic},
	prelude::*,
};
use avail_rust_core::{
	avail::balances::types::AccountData,
	rpc::{self, AllowedExtrinsic},
};
use std::str::FromStr;

#[tokio::test]
pub async fn submit_works() -> Result<(), Error> {
	let client = Client::connect("").await?;
	account_test(&client).await?;
	block_information_test(&client).await?;
	block_extrinsics_test(&client).await?;

	Ok(())
}

// fn hex_encode(value: &[u8]) -> String {
// 	const_hex::encode_prefixed(value)
// }

fn hex_decode(value: &str) -> Vec<u8> {
	const_hex::decode(&value).unwrap()
}

fn h256(value: &str) -> H256 {
	H256::from_str(value).unwrap()
}

pub async fn block_extrinsics_test(client: &Client) -> Result<(), Error> {
	let at = 45838;
	let block: block::Block = client.block(at);
	let query = block.extrinsics();

	assert_eq!(query.count(None, Default::default()).await?, 4);
	assert_eq!(query.exists(None, Default::default()).await?, true);
	let allowed = Some(vec![AllowedExtrinsic::TxIndex(100)]);
	assert_eq!(query.exists(allowed, Default::default()).await?, false);

	let extrinsics = query.all(None, Default::default()).await?;
	assert_eq!(extrinsics.len(), 4);

	// Ext 0
	let expected_0 = UntypedExtrinsic {
		preamble: avail_rust_core::Preamble::Bare(5),
		call: hex_decode("0x03000b6175f6d19c01"),
		metadata: BlockExtrinsicMetadata {
			ext_hash: h256("0x4e440d66dd293259e28cbf55a87ac371ccfb6be336859934d21288134458a0e7"),
			ext_index: 0,
			pallet_id: 3,
			variant_id: 0,
			at: at.into(),
		},
	};
	assert_eq!(extrinsics[0], expected_0);

	// Ext 1
	let expected_1 = UntypedExtrinsic {
		preamble: avail_rust_core::Preamble::Bare(5),
		call: hex_decode("0x060300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d13000064a7b3b6e00d"),
		metadata: BlockExtrinsicMetadata {
			ext_hash: h256("0xcb26cd186e3d0df1c62701b9bde19b77c3ba2d64a2598d8f0948f5301bfff3a8"),
			ext_index: 1,
			pallet_id: 6,
			variant_id: 3,
			at: at.into(),
		},
	};

	// TODO Properly test preamble
	assert_eq!(extrinsics[1].ss58_address(), Some("5F6YY4yp4kqhAdsHLNiCLfK7Msh5oEffFb3xu4eU67HVapqY".to_owned()));
	assert_eq!(extrinsics[1].nonce(), Some(0));
	assert_eq!(extrinsics[1].tip(), Some(0));
	assert_eq!(extrinsics[1].call, expected_1.call);
	assert_eq!(extrinsics[1].metadata, expected_1.metadata);

	// Ext 2
	let expected_2 = UntypedExtrinsic {
		preamble: avail_rust_core::Preamble::Bare(4),
		call: hex_decode("0x1d0600000000000000000000000000"),
		metadata: BlockExtrinsicMetadata {
			ext_hash: h256("0x0afb112c764a4678309dd935dbffde52b0c4a0ce0aca3201cc2f112aee777716"),
			ext_index: 2,
			pallet_id: 29,
			variant_id: 6,
			at: at.into(),
		},
	};
	assert_eq!(extrinsics[2], expected_2);

	// Ext 3
	let expected_3 = UntypedExtrinsic {
		preamble: avail_rust_core::Preamble::Bare(4),
		call: hex_decode("0x270b00"),
		metadata: BlockExtrinsicMetadata {
			ext_hash: h256("0x92cdb77314063a01930b093516d19a453399710cc8ae635ff5ab6cf76b26f218"),
			ext_index: 3,
			pallet_id: 39,
			variant_id: 11,
			at: at.into(),
		},
	};
	assert_eq!(extrinsics[3], expected_3);

	// RPC extrinsics
	let rpc_exts = query
		.rpc(None, Default::default(), DataFormat::Extrinsic)
		.await
		.unwrap();
	assert_eq!(rpc_exts.len(), 4);

	// Rpc Ext 0
	let expected = rpc::Extrinsic {
		data: "280503000b6175f6d19c01".to_owned(),
		ext_hash: expected_0.metadata.ext_hash,
		ext_index: expected_0.metadata.ext_index,
		pallet_id: expected_0.metadata.pallet_id,
		variant_id: expected_0.metadata.variant_id,
		account_id: None,
		nonce: None,
	};
	assert_eq!(rpc_exts[0], expected);

	// Rpc Ext 1
	let expected: rpc::Extrinsic = rpc::Extrinsic {
		data: "4d028400861bd0a1c987fa52beed80f771826ec07c2e3584f3b9d9fb898410dbc7d3641001aad8d94ae3f291efef7887532ed6e1c57c4db74912a63c295243289c824c0c2d3e3336a1f904eecb243061b21453dff73185996243eb5ff565b8432ed138e782a5000000060300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d13000064a7b3b6e00d".to_owned(),
		ext_hash: expected_1.metadata.ext_hash,
		ext_index: expected_1.metadata.ext_index,
		pallet_id: expected_1.metadata.pallet_id,
		variant_id: expected_1.metadata.variant_id,
		account_id: Some(AccountId::from_str("5F6YY4yp4kqhAdsHLNiCLfK7Msh5oEffFb3xu4eU67HVapqY").unwrap()),
		nonce: Some(0),
	};
	assert_eq!(rpc_exts[1], expected);

	// Rpc Ext 2
	let expected: rpc::Extrinsic = rpc::Extrinsic {
		data: "40041d0600000000000000000000000000".to_owned(),
		ext_hash: expected_2.metadata.ext_hash,
		ext_index: expected_2.metadata.ext_index,
		pallet_id: expected_2.metadata.pallet_id,
		variant_id: expected_2.metadata.variant_id,
		account_id: None,
		nonce: None,
	};
	assert_eq!(rpc_exts[2], expected);

	// Rpc Ext 3
	let expected: rpc::Extrinsic = rpc::Extrinsic {
		data: "1004270b00".to_owned(),
		ext_hash: expected_3.metadata.ext_hash,
		ext_index: expected_3.metadata.ext_index,
		pallet_id: expected_3.metadata.pallet_id,
		variant_id: expected_3.metadata.variant_id,
		account_id: None,
		nonce: None,
	};
	assert_eq!(rpc_exts[3], expected);

	// First & Last
	assert_eq!(query.first(None, Default::default()).await?.unwrap(), expected_0);
	assert_eq!(query.last(None, Default::default()).await?.unwrap(), expected_3);

	Ok(())
}

pub async fn block_information_test(client: &Client) -> Result<(), Error> {
	let info = client.chain().info().await?;
	assert!(info.best_height > info.finalized_height);
	assert_eq!(
		info.genesis_hash,
		H256::from_str("0x09281d844f923a241d1e7ccfb4282098b0a2011f686cfebce4c2a2d474feb834").unwrap()
	);

	// Checking Best height/hash
	let pairs = [
		(info.best_hash, info.best_height),
		(info.finalized_hash, info.finalized_height),
	];
	for (hash, height) in pairs {
		assert_eq!(client.chain().block_hash(Some(height)).await?, Some(hash));
		assert_eq!(client.chain().block_height(hash).await?, Some(height));
	}

	let at = 45838;
	let block: block::Block = client.block(at);
	let ts_1 = client.chain().block_timestamp(at).await?;
	let ts_2 = block.timestamp().await?;
	let block_weight = client.chain().block_weight(at).await?;
	let ext_weight = block.extrinsic_weight().await?;
	let author_1 = client.chain().block_author(at).await?;
	let author_2 = block.author().await?;
	let event_count_1 = client.chain().block_event_count(at).await?;
	let event_count_2 = block.event_count().await?;
	let extrinsic_count = block.extrinsic_count().await?;
	let header_1 = client.chain().block_header(Some(at)).await?.unwrap();
	let header_2 = block.header().await?;
	let block_info = block.info().await?;
	let justification = block.justification().await?;

	let total_weight = block_weight.total_weight();
	assert_eq!(ts_1, ts_2);
	assert_eq!(ts_2, 1773049116001);
	assert_eq!(event_count_1, event_count_2);
	assert_eq!(event_count_2, 9);
	assert_eq!(extrinsic_count, 4);
	assert_eq!(author_1.to_string(), author_2.to_string());
	assert_eq!(author_2.to_string(), "5EseWFKtQyQCYYchaepYtkGbgKLhzrAbo9qQ9KczBfF5WURW");
	assert_eq!(total_weight, 54553175162);
	assert_eq!(ext_weight.ref_time, 43626846162);
	assert_eq!(block_info.height, at);
	assert_eq!(
		block_info.hash,
		H256::from_str("0x3d8f0f51c513679550f0d2dabec38ce215370aeac648c0b76a99b2e48181aa5c").unwrap()
	);
	assert_eq!(header_1.number, header_2.number);
	assert_eq!(header_2.number, at);
	assert_eq!(justification.is_none(), true);
	assert_eq!(client.block(1).justification().await?.is_some(), true);

	Ok(())
}

pub async fn account_test(client: &Client) -> Result<(), Error> {
	const ADDRESS: &str = "5FjpzwicaDNAUyFjGGqCx8Ty8ooi5T3zPrziEz2DFcqR42Cp";

	// Accounts
	let balance_1 = client.account().balance(ADDRESS, BlockQueryMode::Finalized).await?;
	let balance_2 = client.account().balance_at(ADDRESS, 45511).await?;
	let info_1 = client.account().info(ADDRESS, BlockQueryMode::Finalized).await?;
	let info_2 = client.account().info_at(ADDRESS, 45511).await?;
	let nonce_1 = client.account().nonce(ADDRESS, BlockQueryMode::Finalized).await?;
	let nonce_2 = client.account().nonce_at(ADDRESS, 45511).await?;

	// all balances must be equal;
	let expected = AccountData {
		free: 6009733929969200839,
		reserved: 0,
		frozen: 0,
		flags: 170141183460469231731687303715884105728,
	};
	assert_eq!(balance_1, balance_2);
	assert_eq!(balance_2, info_1.data);
	assert_eq!(info_1.data, info_2.data);
	assert_eq!(info_2.data, expected);

	// all nonces must be equal;
	let expected = 3u32;
	assert_eq!(nonce_1, nonce_2);
	assert_eq!(nonce_2, info_1.nonce);
	assert_eq!(info_1.nonce, info_2.nonce);
	assert_eq!(info_2.nonce, expected);

	Ok(())
}
