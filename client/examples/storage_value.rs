use avail_rust_client::prelude::*;
use avail_rust_core::avail::data_availability::storage::NextAppId;

// Custom Storage
// For simple storage your type should implement StorageValue
pub struct Now;
impl StorageValue for Now {
	type VALUE = u64;

	const PALLET_NAME: &str = "Timestamp";
	const STORAGE_NAME: &str = "Now";
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;
	let rpc_client = &client.rpc_client;

	// Fetching DataAvailability::NextAppId - Storage Value
	let next_app_id = NextAppId::fetch(rpc_client, None).await?.expect("Should be there");
	println!("DataAvailability::NextAppId: {}", next_app_id.0);

	// Fetching Timestamp::Now - Storage Value
	let now = Now::fetch(&rpc_client, None).await?.expect("Should be there");
	println!("Timestamp::Now: {}", now);

	Ok(())
}

/*
	Expected Output:

	DataAvailability::NextAppId: 499
	Timestamp::Now: 1761827960000
*/
