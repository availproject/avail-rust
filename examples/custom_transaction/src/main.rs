//! This example showcases the following actions:
//! - Creating custom transaction and submitting it
//! - Decoding custom transaction
//!

use avail_rust_client::{
	avail::{TransactionCallLike, TxDispatchIndex},
	avail_rust_core::rpc::system::fetch_extrinsics_v1_types::EncodeSelector,
	prelude::*,
};

#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomTransaction {
	pub data: Vec<u8>,
}
impl TxDispatchIndex for CustomTransaction {
	const DISPATCH_INDEX: (u8, u8) = (29u8, 1u8);
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;
	let custom_tx = CustomTransaction { data: vec![0, 1, 2, 3] };
	let submittable = custom_tx.to_submittable(client.clone());
	let submitted = submittable.sign_and_submit(&alice(), Options::new(Some(2))).await?;
	let receipt = submitted.receipt(true).await?.expect("Must be there");

	let block_tx_1 = block_transaction_from_system_rpc(&client, receipt.clone()).await?;
	let custom_tx_decoded = CustomTransaction::from_ext(&block_tx_1).expect("Must be fromable");
	if custom_tx_decoded.as_ref().ne(&custom_tx) {
		return Err("Created and decoded transaction are not the same".into());
	}

	let block_tx_2 = block_transaction_from_block_rpc(&client, receipt.clone()).await?;
	let custom_tx_decoded = CustomTransaction::from_ext(&block_tx_2).expect("Must be fromable");
	if custom_tx_decoded.as_ref().ne(&custom_tx) {
		return Err("Created and decoded transaction are not the same".into());
	}

	println!("Both created and decoded transaction is the same");

	Ok(())
}

async fn block_transaction_from_system_rpc(
	client: &Client,
	receipt: TransactionReceipt,
) -> Result<Vec<u8>, ClientError> {
	let block_client = client.block_client();
	let block_tx = block_client
		.block_transaction(
			receipt.block_loc.into(),
			receipt.tx_loc.into(),
			None,
			Some(EncodeSelector::Call),
		)
		.await?;
	let block_tx = block_tx.expect("Must be there");

	let encoded_call = block_tx.encoded.expect("Must be there");
	let encoded_call = hex::decode(encoded_call.trim_start_matches("0x")).expect("Must work");
	Ok(encoded_call)
}
async fn block_transaction_from_block_rpc(
	client: &Client,
	receipt: TransactionReceipt,
) -> Result<Vec<u8>, ClientError> {
	let block_client = client.block_client();
	let block = block_client
		.rpc_block(receipt.block_loc.into())
		.await?
		.expect("Must be there");
	let encoded_ext = block
		.extrinsics
		.get(receipt.tx_loc.index as usize)
		.expect("Must be there");
	let Ok(OpaqueTransaction { signature: _, call }) = OpaqueTransaction::try_from(encoded_ext) else {
		return Err("Failed to covert extrinsic to opaque".into());
	};

	Ok(call)
}
