use avail_rust_client::{
	block::{Events, SignedExtrinsics},
	error::Error,
	prelude::*,
};
use avail_rust_core::{
	avail::multisig::{
		events::{MultisigApproval, MultisigCancelled, MultisigExecuted, NewMultisig},
		tx::{ApproveAsMulti, AsMulti, CancelAsMulti},
		types::Timepoint,
	},
	types::substrate::Weight,
};
use codec::Encode;

pub async fn run_tests() -> Result<(), Error> {
	tx_tests().await?;
	event_test().await?;

	Ok(())
}
pub async fn tx_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// ApproveAsMulti
	{
		let block = SignedExtrinsics::new(client.clone(), 1824125);

		let signatures = vec![
			"0xa26556769ad6581b7beb103590a5c378955244aa349bbacc2f148c51205e055a",
			"0xdc5d106accefeea0645567b92a5d1667bfabc834bbab673818956b1c29832c29",
		];
		let call_hash = "0xa4b1ac085cea36f1090309159e91d8468b223a8e77026cb545f285658ec17332";
		let weight = Weight { proof_size: 11037, ref_time: 10625088299 };
		let submittable = client
			.tx()
			.multisig()
			.approve_as_multi(2, signatures, None, call_hash, weight);
		let expected_call = ApproveAsMulti::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<ApproveAsMulti>(5).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// AsMulti
	{
		let block = SignedExtrinsics::new(client.clone(), 1814842);

		let signatures = vec![
			"0x2a960c22ebf8069f53172a91f5754c184e89c87e8435976415ab8c9dd4f0b61c",
			"0x705bfe5b162d54d51808ca5d74094fa72bfaec830f5b1206d8cfd8b6317e7572",
			"0x78459404abf0a6d264c957f113bfd45159d9139692e2680f9670eb95f31eaa6e",
			"0xda6ae7403cf319cde30cc7c3928c444f06ad7f3c69296272e34d225b151c8f6b",
		];
		let call = client.tx().balances().transfer_keep_alive(
			"0x8893040a40f0a275e28e0c15dc9f05144b89771e56f901a0235ebe21c44a36bf",
			50000000000000000000000000,
		);
		let timepoint = Some(Timepoint { height: 1814743, index: 2 });
		let weight = Weight { proof_size: 3593, ref_time: 196085000 };
		let submittable = client.tx().multisig().as_multi(3, signatures, timepoint, call, weight);
		let expected_call = AsMulti::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<AsMulti>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// CancelAsMulti
	{
		let block = SignedExtrinsics::new(client.clone(), 1824115);

		let signatures = vec![
			"0xa26556769ad6581b7beb103590a5c378955244aa349bbacc2f148c51205e055a",
			"0xdc5d106accefeea0645567b92a5d1667bfabc834bbab673818956b1c29832c29",
		];
		let timepoint = Timepoint { height: 1824112, index: 1 };
		let call_hash = "0xd359983366d5cf17ca06bfd071bf514e80ecb05f24ada11e5dead0d3d3f68ee4";
		let submittable = client
			.tx()
			.multisig()
			.cancel_as_multi(2, signatures, timepoint, call_hash);
		let expected_call = CancelAsMulti::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<CancelAsMulti>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	Ok(())
}
pub async fn event_test() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// NewMultisig
	{
		let block = Events::new(client.clone(), 1861590);
		let events = block.ext(1).await?.unwrap();

		let expected = NewMultisig {
			approving: AccountId::from_str("0x4c4062701850428210b0bb341c92891c2cd8f67c5e66326991f8ee335de2394a")
				.unwrap(),
			multisig: AccountId::from_str("0x248fa9bcba295608e1a3d36455a536ac4e4011e8366d8f56effb732b30dc372b")
				.unwrap(),
			call_hash: H256::from_str("0x69aaac7a36fa01d8c5aa1f634490bf4601891dd7ff19ade0787a37016b9d519a").unwrap(),
		};
		let actual = events.first::<NewMultisig>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// MultisigExecuted
	{
		let block = Events::new(client.clone(), 1861592);
		let events = block.ext(1).await?.unwrap();

		let expected = MultisigExecuted {
			approving: AccountId::from_str("0xcf3cb26493846a0a5b758174dbc4dc3f42bf883bc50c8d5f4b4a4d1264dd908e")
				.unwrap(),
			timepoint: Timepoint { height: 1861590, index: 1 },
			multisig: AccountId::from_str("0x248fa9bcba295608e1a3d36455a536ac4e4011e8366d8f56effb732b30dc372b")
				.unwrap(),
			call_hash: H256::from_str("0x69aaac7a36fa01d8c5aa1f634490bf4601891dd7ff19ade0787a37016b9d519a").unwrap(),
			result: Ok(()),
		};
		let actual = events.first::<MultisigExecuted>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// MultisigApproval
	{
		let block = Events::new(client.clone(), 1805938);
		let events = block.ext(1).await?.unwrap();

		let expected = MultisigApproval {
			approving: AccountId::from_str("0xde54c7f5dbab3620e3093ee263983c0d77bc73e0a5a38391b778c99d2f23d60b")
				.unwrap(),
			timepoint: Timepoint { height: 1802555, index: 1 },
			multisig: AccountId::from_str("0x0050e994d5891122c2a3416676cd7c1919b88344ea4fd3fb37ff0c5e6c17d753")
				.unwrap(),
			call_hash: H256::from_str("0xd581a9058842255005b89eb34d85a8631a155b4a8a4aff7d870f544bee5404a3").unwrap(),
		};
		let actual = events.first::<MultisigApproval>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// MultisigCancelled
	{
		let block = Events::new(client.clone(), 1861588);
		let events = block.ext(1).await?.unwrap();

		let expected = MultisigCancelled {
			cancelling: AccountId::from_str("0x4c4062701850428210b0bb341c92891c2cd8f67c5e66326991f8ee335de2394a")
				.unwrap(),
			timepoint: Timepoint { height: 1861566, index: 1 },
			multisig: AccountId::from_str("0x248fa9bcba295608e1a3d36455a536ac4e4011e8366d8f56effb732b30dc372b")
				.unwrap(),
			call_hash: H256::from_str("0x69aaac7a36fa01d8c5aa1f634490bf4601891dd7ff19ade0787a37016b9d519a").unwrap(),
		};
		let actual = events.first::<MultisigCancelled>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	Ok(())
}
