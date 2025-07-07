use avail_rust_client::{
	avail_rust_core::{StorageDoubleMap, StorageHasher, StorageMap, StorageValue},
	clients::storage_client::{
		StorageDoubleMapFetcher, StorageDoubleMapIterator, StorageMapFetcher, StorageMapIterator, StorageValueFetcher,
	},
	prelude::*,
};

pub struct TimestampNow;

impl StorageValue for TimestampNow {
	const PALLET_NAME: &str = "Timestamp";
	const STORAGE_NAME: &str = "Now";
	type VALUE = u64;
}

pub struct DataAvailabilityAppKeys;

#[derive(Debug, Clone, codec::Decode)]
pub struct AppKey {
	pub owner: AccountId,
	#[codec(compact)]
	pub id: u32,
}

impl StorageMap for DataAvailabilityAppKeys {
	const PALLET_NAME: &str = "DataAvailability";
	const STORAGE_NAME: &str = "AppKeys";
	const KEY_HASHER: StorageHasher = StorageHasher::Blake2_128Concat;
	type KEY = Vec<u8>;
	type VALUE = AppKey;
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

	// 468

	let res = StorageDoubleMapFetcher::<StakingErasValidatorPrefs>::fetch(
		&client,
		468,
		AccountId::from_str("5EFs6TqF2knsBtEC6gr5F1cV85N9hkkb2MFuzbEf9zmNMnNV")?,
		Some(block_hash),
	)
	.await?;

	dbg!(res);

	/* 	let mut it = StorageDoubleMapIterator::<StakingErasValidatorPrefs>::new(client, 468, block_hash);
	loop {
		let new = it.next().await?;
		let Some(new) = new else {
			break;
		};

		dbg!(new);
	}

	dbg!(it.next().await?);
	dbg!(it.next().await?); */

	/* 	let value = StorageMapFetcher::<DataAvailabilityAppKeys>::fetch(&client, vec![b'a'], None).await?;
	   dbg!(value);
	*/
	/* 	let mut it = StorageMapIterator::<DataAvailabilityAppKeys>::new(client.clone(), block_hash);
	while let value = it.next().await? {
		if let Some(value) = value.as_ref() {
			dbg!(value.id);
		}
		if value.is_none() {
			break;
		}
	}

	dbg!(it.next().await?);
	dbg!(it.next().await?);
	dbg!(it.next().await?); */

	Ok(())
}
