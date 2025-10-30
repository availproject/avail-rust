use avail_rust_client::prelude::*;
use avail_rust_core::avail::data_availability::storage::AppKeys;

// Custom Storage Map
pub struct SetIdSession;
impl StorageMap for SetIdSession {
	type KEY = u64;
	type VALUE = u32;

	const KEY_HASHER: StorageHasher = StorageHasher::Twox64Concat;
	const PALLET_NAME: &str = "Grandpa";
	const STORAGE_NAME: &str = "SetIdSession";
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;
	let rpc_client = &client.rpc_client;

	// Fetching DataAvailability::AppKeys - Storage Map
	let key = "Hello World".as_bytes().to_vec();
	let app_key = AppKeys::fetch(rpc_client, &key, None).await?.expect("Should be there");
	println!("AppKey Owner: {}, AppKey Id: {}", app_key.owner, app_key.id);
	println!("");

	// Iterating over AppKeys
	let block_hash = client.finalized().block_hash().await?;
	let mut iter = AppKeys::iter(rpc_client.clone(), block_hash);
	for _ in 0..2 {
		// You can fetch just the value...
		let app_key = iter.next().await?.expect("Should be there");
		println!("AppKey Owner: {}, AppKey Id: {}", app_key.owner, app_key.id);

		// ...or both the value and the key
		let (key, app_key) = iter.next_key_value().await?.expect("Should be there");
		println!(
			"AppKey Key: {}, AppKey Owner: {}, AppKey Id: {}",
			String::from_utf8(key).expect("Should work"),
			app_key.owner,
			app_key.id
		)
	}
	println!("");

	// Fetching Grandpa::SetIdSession - Storage Map
	let block_hash = client.chain().block_hash(Some(2504346)).await;
	let block_hash = block_hash?.expect("Should be there");

	let session_index = SetIdSession::fetch(rpc_client, &615, Some(block_hash))
		.await?
		.expect("Should be there");
	println!("Session Index: {:?}", session_index);
	println!("");

	// Iterating over SetIdSession
	let mut iter = SetIdSession::iter(rpc_client.clone(), block_hash);
	for _ in 0..2 {
		// You can fetch just the value...
		let session_index = iter.next().await?.expect("Should be there");
		println!("Session Index: {}", session_index);

		// ...or both the value and the key
		let (set_id, session_index) = iter.next_key_value().await?.expect("Should be there");
		println!("Set Id: {}, Session Index: {}", set_id, session_index)
	}

	Ok(())
}

/*
	Expected Output:

	AppKey Owner: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY, AppKey Id: 237

	AppKey Owner: 5EPRhv5CfHRWujoTMvBg58oeed8QRe4oCqHL4xtsyhY27xCo, AppKey Id: 254
	AppKey Key: My nasasasas sd sd sa ew application key, AppKey Owner: 5CqgQkrDcdg5QrtuxT3H7WszrqgrBMhdwRbmMVXQzc4VSiEg, AppKey Id: 309
	AppKey Owner: 5CocSpj62xG11MAkQMzsd7h8wRoxx1E44f8tJUUrPaWQ4opj, AppKey Id: 18
	AppKey Key: xuelan-avail-cli, AppKey Owner: 5FTZeZ1Gp81952gxXjcyDdzJmrsWjLZkFDESTHVgeKACLb66, AppKey Id: 448

	Session Index: 3201

	Session Index: 3201
	Set Id: 504, Session Index: 2553
	Session Index: 2649
	Set Id: 525, Session Index: 2673
*/
