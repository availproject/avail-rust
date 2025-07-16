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

	println!("Singe Block Transaction");
	block_transaction(client.clone(), receipt.block_loc.into(), receipt.tx_loc.into()).await?;
	println!("Multiple Block Transactions");
	block_transactions(client.clone(), receipt.block_loc.into()).await?;
	println!("RPC Block Transactions");
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
			"ss58: {:?}, Nonce: {}, App Id: {}",
			signature.ss58_address, signature.nonce, signature.app_id
		);
	}

	decode_transaction_call(&info.encoded.expect("Must be there").trim_start_matches("0x"));

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
			"ss58: {:?}, Nonce: {}, App Id: {}",
			signature.ss58_address, signature.nonce, signature.app_id
		);
	}

	decode_transaction(info.encoded.expect("Must be there").trim_start_matches("0x"));

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
				"SS58 Address: {:?}, Nonce: {}, App Id: {}",
				signature.ss58_address, signature.nonce, signature.app_id
			);
		}

		decode_transaction_call(&info.encoded.expect("Must be there").trim_start_matches("0x"));
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
		decode_transaction_bytes(transaction);
	}

	Ok(())
}

pub fn decode_transaction(transaction: &str) {
	// To decode a Transaction you need to do the following:
	// - Hex decode it
	// - SCALE decode it
	// - Decode Transaction Call (Optional)

	// The first option is to directly call `decode_hex_transaction` and convert the
	// Hex and SCALE encoded Transaction to the correct Transaction Call variant.
	//
	// This does the following: OpaqueTransaction::try_from + Self::decode_call
	if let Some(decoded) = SubmitData::decode_hex_transaction(&transaction) {
		println!("Data: {:?}", decoded.data);
	}

	// The second option is to convert the transaction to OpaqueTransaction
	// This gives us access to Transaction Signature and SCALE encoded Transaction Call
	//
	// This does the following: Hex::decode + SCALE::decode
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
				"SS58 Address: {}, Nonce: {}, App Id: {}",
				account_id, signature.tx_extra.nonce, signature.tx_extra.app_id
			);
		};
	}

	// We ca use "Self::decode_call" to SCALE decode the transaction call
	if let Some(decoded) = SubmitData::decode_call(&transaction.call) {
		println!("Data: {:?}", decoded.data);
	}
}

pub fn decode_transaction_bytes(transaction: &[u8]) {
	// To decode a Transaction you need to do the following:
	// - SCALE decode it
	// - Decode Transaction Call (Optional)

	// The first option is to directly call `decode_transaction` and convert the
	// SCALE encoded Transaction to the correct Transaction Call variant.
	//
	// This does the following: OpaqueTransaction::try_from + Self::decode_call
	if let Some(decoded) = SubmitData::decode_transaction(&transaction) {
		println!("Data: {:?}", decoded.data);
	}

	// The second option is to convert the transaction to OpaqueTransaction
	// This gives us access to Transaction Signature and SCALE encoded Transaction Call
	//
	// This does the following: SCALE::decode
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
				"SS58 Address: {}, Nonce: {}, App Id: {}",
				account_id, signature.tx_extra.nonce, signature.tx_extra.app_id
			);
		};
	}

	// We ca use "Self::decode_call" to SCALE decode the transaction call
	if let Some(decoded) = SubmitData::decode_call(&transaction.call) {
		println!("Data: {:?}", decoded.data);
	}
}

pub fn decode_transaction_call(call: &str) {
	// To decode a Transaction Call you need to do the following:
	// - Hex decode it
	// - SCALE decode it

	// The easiest way is to call `Self::decode_hex_call` which will Hex and
	// SCALE decode the call for us
	if let Some(decoded) = SubmitData::decode_hex_call(call) {
		println!("Data: {:?}", decoded.data);
	}

	// Second option is to manually Hex decode the call and call `Self::decode_call`
	// which will SCALE decode the call for us
	let hex_decoded = hex::decode(call).expect("Must work");
	if let Some(decoded) = SubmitData::decode_call(&hex_decoded) {
		println!("Data: {:?}", decoded.data);
	}
}
