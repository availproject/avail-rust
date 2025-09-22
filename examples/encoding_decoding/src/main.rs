//! This example showcases the following actions:
//! - Encode and Decode Transactions
//! - Encode and Decode Events
//! - Encode and Decode Storage
//!

use avail_rust_client::prelude::*;
use codec::Encode;

// Transaction Definition
#[derive(codec::Decode, codec::Encode)]
pub struct CustomTransaction {
	pub data: Vec<u8>,
}
impl HasHeader for CustomTransaction {
	const HEADER_INDEX: (u8, u8) = (29u8, 1u8);
}

// Event Definition
#[derive(codec::Decode, codec::Encode)]
pub struct CustomEvent {
	pub who: AccountId,
	pub data_hash: H256,
}
impl HasHeader for CustomEvent {
	const HEADER_INDEX: (u8, u8) = (29, 1);
}

// Storage Definition
pub struct CustomStorage;
impl StorageMap for CustomStorage {
	type KEY = Vec<u8>;
	type VALUE = AppKey;

	const KEY_HASHER: StorageHasher = StorageHasher::Blake2_128Concat;
	const PALLET_NAME: &str = "DataAvailability";
	const STORAGE_NAME: &str = "AppKeys";
}
#[derive(Debug, Clone, codec::Decode, codec::Encode)]
pub struct AppKey {
	pub owner: AccountId,
	#[codec(compact)]
	pub id: u32,
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	transaction_decoding_encoding().await?;
	storage_decoding_encoding();
	event_decoding_encoding();

	Ok(())
}

async fn transaction_decoding_encoding() -> Result<(), ClientError> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	// Decoding

	// Decoding Hex Transaction Call to our Custom Transaction
	// For decoding from bytes call `decode_call`
	let ct = CustomTransaction::decode_hex_call("0x1d010c616263").expect("Should work");
	assert_eq!(ct.data, vec![b'a', b'b', b'c']);

	// Decoding whole Hex Transaction to our Custom Transaction
	// For decoding from bytes call `decode_transaction`
	let tx = "0xb9018400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d014ac740fa88d56954b4a3982e0fa9cdc8f44d8364c01fefc895c7751395709c1fda59696f4b9b74e1831e92487e62122cb4ac3ec82aa1af52a4473866f29dc087150104000c1d010c616263";
	let ct = CustomTransaction::decode_hex_transaction(tx).expect("Should work");
	assert_eq!(ct.data, vec![b'a', b'b', b'c']);

	// Decoding whole Hex Transaction to Opaque Transaction and then to Custom Transaction
	// `try_from` accepts &[u8] as well
	let opaq = OpaqueTransaction::try_from(tx).expect("Should work");
	let sig = opaq.signature.as_ref().expect("qed");
	assert_eq!(sig.tx_extra.app_id, 3);
	let ct = CustomTransaction::decode_call(&opaq.call).expect("Should work");
	assert_eq!(ct.data, vec![b'a', b'b', b'c']);

	// Decoding whole Hex Transaction to Decoded Transaction
	// `try_from` accepts &[u8] as well
	let decoded_tx = DecodedTransaction::<CustomTransaction>::try_from(tx).expect("Should work");
	let sig = decoded_tx.signature.as_ref().expect("qed");
	assert_eq!(sig.tx_extra.app_id, 3);
	assert_eq!(decoded_tx.call.data, vec![b'a', b'b', b'c']);

	// Encoding
	let custom = CustomTransaction { data: vec![0, 1, 2] };
	let submittable = custom.to_submittable(client.clone());

	// Submitting
	let submitted = submittable.sign_and_submit(&alice(), Options::new(2)).await?;
	let receipt = submitted.receipt(true).await?.expect("Must be there");
	println!("Block Hash: {:?}", receipt.block_ref.hash);

	Ok(())
}

fn event_decoding_encoding() {
	let target_who = AccountId::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").expect("Must Work");
	let target_data_hash =
		H256::from_str("0xbddd813c634239723171ef3fee98579b94964e3bb1cb3e427262c8c068d52319").expect("Must work");

	// Decoding
	let event = "0x1d01d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27dbddd813c634239723171ef3fee98579b94964e3bb1cb3e427262c8c068d52319";
	let custom = CustomEvent::decode_hex_event(event).expect("Must Work");
	assert_eq!(target_who, custom.who);
	assert_eq!(target_data_hash, custom.data_hash);

	// Encoding
	let custom = CustomEvent { who: target_who, data_hash: target_data_hash };
	assert_eq!(event, custom.encode_as_hex_event());
}

fn storage_decoding_encoding() {
	// Decoding
	let storage_key = "0x905e59f6c8fc974ec64116e6f647992829ac34430c4934c5e3aaeed5abe53e39dda4a166b47f407f37684315c7dfa66414417661696c";
	let storage_value = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00";

	let custom_key = CustomStorage::decode_hex_storage_key(storage_key).expect("Must Work");
	let custom_value = CustomStorage::decode_hex_storage_value(storage_value).expect("Must Work");

	let owner = AccountId::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").expect("Must Work");

	assert_eq!(String::from_utf8(custom_key).expect("Must Work"), String::from("Avail"));
	assert_eq!(custom_value.id, 0);
	assert_eq!(custom_value.owner, owner);

	// Encoding
	let actual_key = CustomStorage::hex_encode_storage_key(&"Avail".as_bytes().to_owned());
	let actual_value = AppKey { id: 0, owner }.encode();

	assert_eq!(storage_key, actual_key.as_str());
	assert_eq!(storage_value, std::format!("0x{}", const_hex::encode(actual_value)));
}
