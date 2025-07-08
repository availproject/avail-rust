use avail_rust_client::{
	avail_rust_core::{StorageDoubleMap, StorageHasher, StorageMap, StorageValue},
	prelude::*,
};

pub struct TimestampNow;
impl StorageValue for TimestampNow {
	const PALLET_NAME: &str = "Timestamp";
	const STORAGE_NAME: &str = "Now";
	type VALUE = u64;
}

pub struct DataAvailabilityAppKeys;
impl StorageMap for DataAvailabilityAppKeys {
	const PALLET_NAME: &str = "DataAvailability";
	const STORAGE_NAME: &str = "AppKeys";
	const KEY_HASHER: StorageHasher = StorageHasher::Blake2_128Concat;
	type KEY = Vec<u8>;
	type VALUE = AppKey;
}
#[derive(Debug, Clone, codec::Decode)]
pub struct AppKey {
	pub owner: AccountId,
	#[codec(compact)]
	pub id: u32,
}

pub struct StakingErasValidatorPrefs;
impl StorageDoubleMap for StakingErasValidatorPrefs {
	const PALLET_NAME: &str = "Staking";
	const STORAGE_NAME: &str = "ErasValidatorPrefs";
	const KEY1_HASHER: StorageHasher = StorageHasher::Twox64Concat;
	const KEY2_HASHER: StorageHasher = StorageHasher::Twox64Concat;
	type KEY1 = u32;
	type KEY2 = AccountId;
	type VALUE = ValidatorPrefs;
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
	Client::enable_tracing(false);
	let client = Client::new(TURING_ENDPOINT).await?;
	let block_hash = client.finalized_block_hash().await.unwrap();

	// Fetching Storage Value
	let value = TimestampNow::fetch(&client.rpc_client, Some(block_hash))
		.await?
		.expect("Needs to be there");
	println!("Timestamp: {}", value);

	// Fetching Storage Map
	let value = DataAvailabilityAppKeys::fetch(
		&client.rpc_client,
		"MyAwesomeKey".to_string().into_bytes(),
		Some(block_hash),
	)
	.await?
	.expect("Needs to be there");
	println!("Owner: {}, id: {}", value.owner, value.id);

	// Iterating Storage Map
	let mut iter = DataAvailabilityAppKeys::iter(client.rpc_client.clone(), block_hash);
	for _ in 0..5 {
		let value = iter.next().await?.expect("Needs to be there");
		println!("Owner: {}, id: {}", value.owner, value.id);

		let (key, value) = iter.next_key_value().await?.expect("Needs to be there");
		println!(
			"Key: {}, Owner: {}, id: {}",
			String::from_utf8(key).expect(""),
			value.owner,
			value.id
		);
	}

	// Fetching Double Storage Map
	let value = StakingErasValidatorPrefs::fetch(
		&client.rpc_client,
		468,
		AccountId::from_str("5EFs6TqF2knsBtEC6gr5F1cV85N9hkkb2MFuzbEf9zmNMnNV")?,
		Some(block_hash),
	)
	.await?
	.expect("Needs to be there");
	println!("Blocked: {}, Commission: {}", value.blocked, value.commission);

	let mut iter = StakingErasValidatorPrefs::iter(client.rpc_client.clone(), 468, block_hash);
	for _ in 0..5 {
		let value = iter.next().await?.expect("Needs to be there");
		println!("Blocked: {}, Commission: {}", value.blocked, value.commission);

		let (key1, key2, value) = iter.next_key_value().await?.expect("Needs to be there");
		println!(
			"Key1: {}, Key2: {}, Blocked: {}, Comission: {}",
			key1, key2, value.blocked, value.commission
		);
	}

	Ok(())
}
