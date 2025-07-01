//! This example showcases the following actions:
//! - Fetching block and block extrinsics via block client
//! - Decoding block extrinsics
//!

use avail::data_availability::tx::Call;
use avail_rust_client::prelude::*;
use avail_rust_core::rpc::system::fetch_extrinsics_v1_types as Types;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let tx = client.tx().data_availability().submit_data(vec![0, 1, 2]);
	let submitted = tx.sign_and_submit(&alice(), Options::new(Some(2))).await?;
	let receipt = submitted.receipt(true).await?.expect("Should be there");

	rpc_block(client.clone(), receipt.block_loc.hash).await?;
	rpc_block_with_justifications(client.clone(), receipt.block_loc.hash).await?;
	block_transaction(client.clone(), receipt.block_loc.hash, receipt.tx_loc.hash).await?;
	block_transactions(client.clone(), receipt.block_loc.hash).await?;
	Ok(())
}

pub async fn block_transaction(client: Client, block_hash: H256, tx_hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	let selector = Some(Types::EncodeSelector::Call);
	let info = blocks
		.block_transaction(HashNumber::Hash(block_hash), HashNumber::Hash(tx_hash), None, selector)
		.await?
		.expect("Should be there");

	decode_transaction_03(&info)?;

	Ok(())
}

pub async fn block_transactions(client: Client, block_hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	let selector = Some(Types::EncodeSelector::Call);
	let params = Types::Params::new(HashNumber::Hash(block_hash), None, selector);
	let infos = blocks.block_transactions(params).await?;
	for info in infos {
		decode_transaction_03(&info)?;
	}

	Ok(())
}

pub async fn rpc_block(client: Client, hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	let block = blocks.rpc_block(hash).await?.expect("Should be there");
	println!("Height: {}", block.header.number);

	for raw_ext in block.extrinsics.iter() {
		// Transactions: OpaqueTransaction + RuntimeCall
		decode_transaction_01(raw_ext);
		// Transactions: DecodedTransaction
		decode_transaction_02(raw_ext);
	}

	Ok(())
}

pub async fn rpc_block_with_justifications(client: Client, hash: H256) -> Result<(), ClientError> {
	let blocks = client.block_client();

	let block_w_just = blocks
		.rpc_block_with_justifications(hash)
		.await?
		.expect("Should be there");
	println!("Height: {}", block_w_just.block.header.number);

	for raw_ext in block_w_just.block.extrinsics.iter() {
		// Transactions: OpaqueTransaction + RuntimeCall
		decode_transaction_01(raw_ext);
		// Transactions: DecodedTransaction
		decode_transaction_02(raw_ext);
	}

	Ok(())
}

pub fn decode_transaction_01(raw_ext: &Vec<u8>) {
	let Ok(opaque_tx) = OpaqueTransaction::try_from(raw_ext) else {
		return;
	};

	println!(
		"Pallet index: {}, Call index: {}",
		opaque_tx.pallet_index(),
		opaque_tx.call_index()
	);

	let Ok(runtime_call) = RuntimeCall::try_from(&opaque_tx.call) else {
		return;
	};

	let RuntimeCall::DataAvailability(Call::SubmitData(sd)) = &runtime_call else {
		return;
	};

	let address = opaque_tx.signature.as_ref().map(|x| x.address.clone()).expect("qed");
	let MultiAddress::Id(account_id) = address else {
		return;
	};

	println!("Address: {}, Submitted data: {:?}", account_id, &sd.data[0..3])
}

pub fn decode_transaction_02(raw_ext: &Vec<u8>) {
	let Ok(decoded_tx) = DecodedTransaction::try_from(raw_ext) else {
		return;
	};

	println!(
		"Pallet index: {}, Call index: {}",
		decoded_tx.pallet_index(),
		decoded_tx.call_index()
	);

	let RuntimeCall::DataAvailability(Call::SubmitData(sd)) = &decoded_tx.call else {
		return;
	};

	let address = decoded_tx.signature.as_ref().map(|x| x.address.clone()).expect("qed");
	let MultiAddress::Id(account_id) = address else {
		return;
	};

	println!("Address: {}, Submitted data: {:?}", account_id, &sd.data[0..3])
}

pub fn decode_transaction_03(info: &Types::ExtrinsicInformation) -> Result<(), ClientError> {
	println!(
		"Tx Hash: {:?}, Tx Index: {}, Pallet Id: {}, Call Id: {}",
		info.tx_hash, info.call_id, info.pallet_id, info.call_id
	);
	println!("Tx Encoded: {:?}", info.encoded);

	if let Some(signature) = &info.signature {
		println!(
			"ss58: {:?}, nonce: {}, app id: {}",
			signature.ss58_address, signature.nonce, signature.app_id
		);
	}

	let Some(encoded) = &info.encoded else {
		return Err("Failed to fetch encoded data".into());
	};
	let encoded = hex::decode(encoded.trim_start_matches("0x")).expect("should be decodable");

	let Ok(runtime_call) = RuntimeCall::try_from(&encoded) else {
		return Ok(());
	};

	dbg!(&runtime_call);

	let RuntimeCall::DataAvailability(Call::SubmitData(sd)) = &runtime_call else {
		return Ok(());
	};

	println!("Data: {:?}", std::format!("0x{}", hex::encode(&sd.data)));

	Ok(())
}
