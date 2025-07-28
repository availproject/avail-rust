//! This example showcases the following actions:
//! - Fetching block and block extrinsics via block client
//! - Decoding block extrinsics
//!
//! Blocks and transactions can be fetched in two ways:
//! - via Block RPC
//! - via Block Transactions RPC
//!
//! The difference between these two RPCs is that the first one will always fetch the whole block
//! (block header + all transactions + justifications) while the second RPC will fetch only the
//! transactions that we specify plus it decodes some of the transaction parts for us.
//!
//! In 99.99% cases transactions RPC is the one that you need/want

use avail::data_availability::tx::SubmitData;
use avail_rust_client::{
	avail_rust_core::rpc::system::fetch_extrinsics_v1_types::{SignatureFilter, TransactionFilter},
	prelude::*,
};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	// Submit Dummy transaction so that we have something to play with
	let receipt = submit_dummy_transactions(&client).await?;

	block_rpc_example(client.clone(), receipt.block_loc.hash).await?;
	transaction_example(client.clone(), receipt.block_loc.into(), receipt.tx_loc.into()).await?;
	transaction_static_example(client.clone(), receipt.block_loc.into(), receipt.tx_loc.into()).await?;
	transactions_example(client.clone(), receipt.block_loc.into()).await?;
	transactions_filter_example(client.clone(), receipt.block_loc.into()).await?;

	Ok(())
}

pub async fn submit_dummy_transactions(client: &Client) -> Result<TransactionReceipt, ClientError> {
	let tx = client.tx().data_availability().submit_data(vec![0, 1, 2]);
	let submitted = tx.sign_and_submit(&alice(), Options::new(Some(2))).await?;
	let receipt = submitted.receipt(true).await?.expect("Should be there");
	Ok(receipt)
}

/// This example showcases how to access and decode the following data:
/// - Block Header
/// - Block Transactions
/// - Block Justifications
/// from an rpc block
///
/// In 99.99% cases `transaction` and `transactions` methods are the one that you need/want
pub async fn block_rpc_example(client: Client, hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	let block_w_justification = blocks.rpc_block(hash).await?.expect("Should be there");
	let block = &block_w_justification.block;

	// Accessing Block Header data
	println!("Block Height: {}", block.header.number);

	// Iterating over Justifications
	if let Some(justifications) = &block_w_justification.justifications {
		for justification in justifications {
			println!("Justification: {:?}", justification);
		}
	}

	// Iterating over All Block Transactions and decoding them
	for transaction in block.extrinsics.iter() {
		decode_transaction_bytes(transaction);
	}

	Ok(())
}

/// This example showcases how to fetch and decode a specific Transaction
pub async fn transaction_example(client: Client, block_hash: H256, tx_hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	// Fetching only the Transaction Call from the block
	let info = blocks
		.transaction(block_hash.into(), tx_hash.into(), None)
		.await?
		.expect("Should be there");

	// Printing out Transaction metadata like: Tx Hash, Tx Index, Pallet Id, Call Id
	println!(
		"Tx Hash: {:?}, Tx Index: {}, Pallet Id: {}, Call Id: {}",
		info.tx_hash, info.call_id, info.pallet_id, info.call_id
	);

	// Printing out Transaction signature data like: Signer, Nonce, App Id
	if let Some(signature) = &info.signature {
		println!("ss58: {:?}, Nonce: {}, App Id: {}", signature.ss58_address, signature.nonce, signature.app_id);
	}

	// Decoding the Transaction Call
	decode_transaction_call(&info.encoded.expect("Must be there"));

	// Fetching the whole transaction from the block
	let info = blocks
		.transaction(block_hash.into(), tx_hash.into(), Some(EncodeSelector::Extrinsic))
		.await?
		.expect("Should be there");

	// Printing out Transaction metadata like: Tx Hash, Tx Index, Pallet Id, Call Id
	println!(
		"Tx Hash: {:?}, Tx Index: {}, Pallet Id: {}, Call Id: {}",
		info.tx_hash, info.call_id, info.pallet_id, info.call_id
	);

	// Printing out Transaction signature data like: Signer, Nonce, App Id
	if let Some(signature) = &info.signature {
		println!("ss58: {:?}, Nonce: {}, App Id: {}", signature.ss58_address, signature.nonce, signature.app_id);
	}

	decode_transaction(&info.encoded.expect("Must be there"));

	Ok(())
}

/// This example showcases how to fetch and decode a specific Transaction
pub async fn transaction_static_example(client: Client, block_hash: H256, tx_hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	// Fetching the whole transaction from a block. This API will automatically decode it as well.
	let (transaction, info) = blocks
		.transaction_static::<SubmitData>(block_hash.into(), tx_hash.into())
		.await?
		.expect("Should be there");

	// Printing out Transaction metadata like: Tx Hash, Tx Index, Pallet Id, Call Id
	println!(
		"Tx Hash: {:?}, Tx Index: {}, Pallet Id: {}, Call Id: {}",
		info.tx_hash, info.call_id, info.pallet_id, info.call_id
	);

	// Printing out Transaction signature data like: Signer, Nonce, App Id
	if let Some(signature) = &info.signature {
		println!("ss58: {:?}, Nonce: {}, App Id: {}", signature.ss58_address, signature.nonce, signature.app_id);
	}

	println!("Data: {:?}", transaction.call.data);

	Ok(())
}

/// This example showcases how to fetch and decode multiple transactions from a block
pub async fn transactions_example(client: Client, block_hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	// This will fetch all block transactions.
	let infos = blocks.transactions(block_hash.into(), None, None, None).await?;
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

		decode_transaction_call(&info.encoded.expect("Must be there"));
	}

	Ok(())
}

/// This example showcases how to filter specific transactions from a block
///
/// To decode the transactions take a look at `transactions_example` example !!!
pub async fn transactions_filter_example(client: Client, block_hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	// This will fetch all block transactions that have App Id set to `2`
	let signature_filter = Some(SignatureFilter::default().app_id(2));
	let infos = blocks
		.transactions(block_hash.into(), None, signature_filter, None)
		.await?;

	assert_eq!(infos.len(), 1);
	for info in infos {
		let signature = &info.signature;
		assert_eq!(signature.as_ref().and_then(|x| Some(x.app_id)), Some(2));
	}

	// This will fetch only block transactions with indices 0 and 1
	let transaction_filter = Some(TransactionFilter::TxIndex(vec![0, 1]));
	let infos = blocks
		.transactions(block_hash.into(), transaction_filter, None, None)
		.await?;
	assert_eq!(infos.len(), 2);
	for (i, info) in infos.iter().enumerate() {
		assert_eq!(info.tx_index, i as u32)
	}

	// This will fetch only block transactions that were submitted by Alice
	let address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string();
	let signature_filter = Some(SignatureFilter::default().ss58_address(address.clone()));
	let infos = blocks
		.transactions(block_hash.into(), None, signature_filter, None)
		.await?;
	assert_eq!(infos.len(), 1);
	for info in infos.iter() {
		let signature = &info.signature;
		assert_eq!(signature.as_ref().and_then(|x| x.ss58_address.clone()), Some(address.clone()));
	}

	Ok(())
}

/// This example showcases how to decode Hex and SCALE encoded Transaction
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
	let transaction = OpaqueTransaction::try_from(transaction).expect("Must work");

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

/// This example showcases how to decode SCALE encoded Transaction
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
	let transaction = OpaqueTransaction::try_from(transaction).expect("Must work");

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

/// This example showcases how to decode Hex and SCALE encoded Transaction Call
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
	let hex_decoded = const_hex::decode(call.trim_start_matches("0x")).expect("Must work");
	if let Some(decoded) = SubmitData::decode_call(&hex_decoded) {
		println!("Data: {:?}", decoded.data);
	}
}
