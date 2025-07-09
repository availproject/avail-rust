//! This example showcases the following actions:
//! - Fetching block and block extrinsics via block client
//! - Decoding block extrinsics
//!

use avail::data_availability::tx::SubmitData;
use avail_rust_client::prelude::*;
use avail_rust_core::rpc::system::fetch_extrinsics_v1_types as Types;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let tx = client.tx().data_availability().submit_data(vec![0, 1, 2]);
	let submitted = tx.sign_and_submit(&alice(), Options::new(Some(2))).await?;
	let receipt = submitted.receipt(true).await?.expect("Should be there");

	block_transaction(client.clone(), receipt.block_loc.hash, receipt.tx_loc.hash).await?;
	block_transactions(client.clone(), receipt.block_loc.hash).await?;
	rpc_block(client.clone(), receipt.block_loc.hash).await?;
	Ok(())
}

pub async fn block_transaction(client: Client, block_hash: H256, tx_hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	// Fetching only the transaction call from the block
	let info = blocks
		.block_transaction(block_hash.into(), tx_hash.into(), None, Some(EncodeSelector::Call))
		.await?
		.expect("Should be there");

	println!(
		"Tx Hash: {:?}, Tx Index: {}, Pallet Id: {}, Call Id: {}",
		info.tx_hash, info.call_id, info.pallet_id, info.call_id
	);
	if let Some(signature) = &info.signature {
		println!(
			"ss58: {:?}, nonce: {}, app id: {}",
			signature.ss58_address, signature.nonce, signature.app_id
		);
	}

	let call = &info.encoded.expect("Must be there");
	let call = hex::decode(call.trim_start_matches("0x")).expect("should be decodable");
	decode_call(&call);

	// Fetching the whole transaction from the block
	let info = blocks
		.block_transaction(block_hash.into(), tx_hash.into(), None, Some(EncodeSelector::Extrinsic))
		.await?
		.expect("Should be there");

	println!(
		"Tx Hash: {:?}, Tx Index: {}, Pallet Id: {}, Call Id: {}",
		info.tx_hash, info.call_id, info.pallet_id, info.call_id
	);
	if let Some(signature) = &info.signature {
		println!(
			"ss58: {:?}, nonce: {}, app id: {}",
			signature.ss58_address, signature.nonce, signature.app_id
		);
	}

	let transaction = &info.encoded.expect("Must be there");
	let transaction = hex::decode(transaction.trim_start_matches("0x")).expect("should be decodable");
	decode_transaction(&transaction);

	Ok(())
}

pub async fn block_transactions(client: Client, block_hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	let params = Some(Types::Options::new(None, Some(EncodeSelector::Call)));
	let infos = blocks.block_transactions(block_hash.into(), params).await?;
	for info in infos {
		println!(
			"Tx Hash: {:?}, Tx Index: {}, Pallet Id: {}, Call Id: {}",
			info.tx_hash, info.call_id, info.pallet_id, info.call_id
		);
		if let Some(signature) = &info.signature {
			println!(
				"SS58 Address: {:?}, nonce: {}, app id: {}",
				signature.ss58_address, signature.nonce, signature.app_id
			);
		}

		let call = &info.encoded.expect("Must be there");
		let call = hex::decode(call.trim_start_matches("0x")).expect("should be decodable");
		decode_call(&call);
	}

	Ok(())
}

pub async fn rpc_block(client: Client, hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	let block_w_justification = blocks.rpc_block(hash).await?.expect("Should be there");
	let block = &block_w_justification.block;
	let justifications = &block_w_justification.justifications;
	println!("Block Height: {}", block.header.number);

	if let Some(justifications) = justifications {
		for justification in justifications {
			println!("Justification: {:?}", justification);
		}
	}

	for transaction in block.extrinsics.iter() {
		decode_transaction(transaction);
	}

	Ok(())
}

pub fn decode_transaction(transaction: &[u8]) {
	let Ok(transaction) = OpaqueTransaction::try_from(transaction) else {
		return;
	};

	println!(
		"Pallet index: {}, Call index: {}, Call length: {}",
		transaction.pallet_index(),
		transaction.call_index(),
		transaction.call.len(),
	);

	if let Some(signature) = &transaction.signature {
		if let MultiAddress::Id(account_id) = &signature.address {
			println!(
				"SS58 Address: {}, Nonce: {}, AppId: {}",
				account_id, signature.tx_extra.nonce, signature.tx_extra.app_id
			);
		};
	}

	decode_call(&transaction.call)
}

pub fn decode_call(call: &[u8]) {
	if let Some(call) = SubmitData::decode_call(&call) {
		println!("Data: {:?}", call.data);
	}
}
