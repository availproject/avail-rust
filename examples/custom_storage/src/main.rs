//! This example showcases the following actions:
//! - Creating custom storage
//! - Decoding custom storage
//!

use avail_rust_client::prelude::*;

pub struct TimestampNow;
impl StorageValue for TimestampNow {
	type VALUE = u64;

	const PALLET_NAME: &str = "Timestamp";
	const STORAGE_NAME: &str = "Now";
}

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

#[derive(Debug, Clone, codec::Encode, codec::Decode)]
pub struct ValidatorPrefs {
	/// Reward that validator takes up-front; only the rest is split between themselves and
	/// nominators.
	#[codec(compact)]
	pub commission: u32,
	/// Whether or not this validator is accepting more nominations. If `true`, then no nominator
	/// who is not already nominating this validator may nominate them. By default, validators
	/// are accepting nominations.
	pub blocked: bool,
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	let client = Client::new(TURING_ENDPOINT).await?;
	let block_hash = client.finalized().block_hash().await.unwrap();

	// Fetching Storage Value
	let value = TimestampNow::fetch(&client.rpc_client, Some(block_hash))
		.await?
		.expect("Needs to be there");
	println!("Timestamp: {}", value);

	// Fetching Storage Map
	let value =
		DataAvailabilityAppKeys::fetch(&client.rpc_client, &"MyAwesomeKey".as_bytes().to_vec(), Some(block_hash))
			.await?
			.expect("Needs to be there");
	println!("Owner: {}, id: {}", value.owner, value.id);

	// Iterating Storage Map
	let mut iter = DataAvailabilityAppKeys::iter(client.rpc_client.clone(), block_hash);
	for _ in 0..5 {
		// You can fetch just the value...
		let value = iter.next().await?.expect("Needs to be there");
		println!("Owner: {}, id: {}", value.owner, value.id);

		// ...or both the value and the key
		let (key, value) = iter.next_key_value().await?.expect("Needs to be there");
		println!("Key: {}, Owner: {}, id: {}", String::from_utf8(key).expect(""), value.owner, value.id);
	}

	// Fetching Double Storage Map
	let value = StakingErasValidatorPrefs::fetch(
		&client.rpc_client,
		&468,
		&AccountId::from_str("5EFs6TqF2knsBtEC6gr5F1cV85N9hkkb2MFuzbEf9zmNMnNV")?,
		Some(block_hash),
	)
	.await?
	.expect("Needs to be there");
	println!("Blocked: {}, Commission: {}", value.blocked, value.commission);

	let mut iter = StakingErasValidatorPrefs::iter(client.rpc_client.clone(), &468, block_hash);
	for _ in 0..5 {
		// You can fetch just the value...
		let value = iter.next().await?.expect("Needs to be there");
		println!("Blocked: {}, Commission: {}", value.blocked, value.commission);

		// ...or both the value and the key
		let (key1, key2, value) = iter.next_key_value().await?.expect("Needs to be there");
		println!("Key1: {}, Key2: {}, Blocked: {}, Comission: {}", key1, key2, value.blocked, value.commission);
	}

	Ok(())
}
