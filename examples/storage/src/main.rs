use avail_rust_client::{
	avail_rust_core::{StorageHasher, StorageMap, StorageValue},
	clients::storage_client::{StorageMapFetcher, StorageMapIterator, StorageValueFetcher},
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

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;
	let block_hash = client.best_block_hash().await.unwrap();

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
