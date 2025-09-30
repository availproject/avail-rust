use avail::staking::types::ValidatorPrefs;
use avail_rust::prelude::*;

// Custom Extrinsic
// Implementing HasHeader, codec::Decode and coded::Encode is
// enough to create a custom extrinsic
#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomExtrinsic {
	pub data: Vec<u8>,
}
impl HasHeader for CustomExtrinsic {
	const HEADER_INDEX: (u8, u8) = (29, 1);
}

// Custom Event
// Implementing HasHeader, codec::Decode and coded::Encode is
// enough to create a custom event
#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomEvent {
	pub who: AccountId,
	pub data_hash: H256,
}
impl HasHeader for CustomEvent {
	const HEADER_INDEX: (u8, u8) = (29, 1);
}

// Custom Storage
// For simple storage your type should implement StorageValue
pub struct TimestampNow;
impl StorageValue for TimestampNow {
	type VALUE = u64;

	const PALLET_NAME: &str = "Timestamp";
	const STORAGE_NAME: &str = "Now";
}

// For map your type should implement StorageMap
pub struct DataAvailabilityAppKeys;
impl StorageMap for DataAvailabilityAppKeys {
	type KEY = Vec<u8>;
	type VALUE = AppKey;

	const KEY_HASHER: StorageHasher = StorageHasher::Blake2_128Concat;
	const PALLET_NAME: &str = "DataAvailability";
	const STORAGE_NAME: &str = "AppKeys";
}
#[derive(Debug, Clone, codec::Decode)]
pub struct AppKey {
	pub owner: AccountId,
	#[codec(compact)]
	pub id: u32,
}

// And for double map your type should implement StorageDoubleMap
pub struct StakingErasValidatorPrefs;
impl StorageDoubleMap for StakingErasValidatorPrefs {
	type KEY1 = u32;
	type KEY2 = AccountId;
	type VALUE = ValidatorPrefs;

	const KEY1_HASHER: StorageHasher = StorageHasher::Twox64Concat;
	const KEY2_HASHER: StorageHasher = StorageHasher::Twox64Concat;
	const PALLET_NAME: &str = "Staking";
	const STORAGE_NAME: &str = "ErasValidatorPrefs";
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Custom Extrinsic
	let block = BlockWithTx::new(client.clone(), 1922190);
	let block_tx = block.get::<CustomExtrinsic>(1).await?.expect("Should be decodable");
	println!("Data Len: {}", block_tx.call.data.len());
	/*
		Data Len: 184379
	*/

	// Custom Event
	let tx_events = block_tx.events(client.clone()).await?;
	let event = tx_events.first::<CustomEvent>().expect("Should be there");
	println!("Who: {}, Data Hash: {:?}", event.who, event.data_hash);
	/*
		Who: 5EZZm8AKzZw8ti9PSmTZdXCgNEeaE3vs5sNxqkQ6u5NhG8kT, Data Hash: 0x94504118a0ce3537d082e821413a172d745b3059dd9c385eceef64933e81aa72
	*/

	// Fetching Custom Simple Storage
	let block_hash = client
		.chain()
		.block_hash(Some(1922190))
		.await?
		.expect("Should be there");
	let now = TimestampNow::fetch(&client.rpc_client, Some(block_hash)).await?;
	let now = now.expect("Should be there");
	println!("Timestamp: {}", now);
	/*
		Timestamp: 1758540340000
	*/

	// Fetching Custom Storage Map
	let key = "kraken".as_bytes().to_vec();
	let value = DataAvailabilityAppKeys::fetch(&client.rpc_client, &key, Some(block_hash)).await?;
	let value = value.expect("Should be there");
	println!("Owner: {}, id: {}", value.owner, value.id);
	/*
		Owner: 5CwFhhZoS2LP5tWAzurDznL949zEemjePH1XBdw6grgyrEPB, id: 41
	*/

	// Iterating Storage Map
	let mut iter = DataAvailabilityAppKeys::iter(client.rpc_client.clone(), block_hash);
	for _ in 0..3 {
		// You can fetch just the value...
		let value = iter.next().await?.expect("Should be there");
		println!("Owner: {}, id: {}", value.owner, value.id);

		// ...or both the value and the key
		let (key, value) = iter.next_key_value().await?.expect("Should be there");
		println!("Key: {}, Owner: {}, id: {}", String::from_utf8(key).expect(""), value.owner, value.id);
	}
	/*
		Owner: 5HW7TTatWztvj48SMuXYMMyrcR3w8upPeHThbrAtkwdey5DP, id: 13
		Key: CRESTAL, Owner: 5H8ffXeTguDWqhtKBG3ygxszjDa52fBovJjndhaMtRGwAcW9, id: 16
		Owner: 5E2QSk9cQ45CCq8PgjrNWNov5yk3q6qAVyNzvwQxUA8avVFb, id: 24
		Key: Lens-Mainnet, Owner: 5C7yF9M6JHza8m8MxF1Ljc1msaaCExuW3pCFsBJSgFZMkrYw, id: 26
		Owner: 5DPYFRuFkJQSA6ipXD2V6sqwjQYV7cS3sCaJ6KpAmgRPiDnA, id: 18
		Key: sophon mainnet , Owner: 5D7GadfhbbkSqCugapLwP1nj6xkTZTEG1UbG11ndzCwEPBJh, id: 36
	*/

	Ok(())
}
