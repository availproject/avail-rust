//! This example showcases the following actions:
//! - Fetching storage
//!

use avail::{data_availability::storage as DAStorage, system::storage as SystemStorage};
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Example with Account Storage
	let account_id = AccountId::from_str("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ").expect("Should work");
	let account_info = SystemStorage::Account::fetch(&client.rpc_client, &account_id, None)
		.await?
		.expect("Should be there");
	println!(
		"Account Nonce: {}, Account Free Balance: {}",
		account_info.nonce, account_info.data.free
	);

	let mut iter = SystemStorage::Account::iter(client.rpc_client.clone(), client.finalized_block_hash().await?);
	for _ in 0..10 {
		// You can fetch just the value...
		let account_info = iter.next().await?.expect("Must be there");
		println!(
			"Account Nonce: {}, Account Free Balance: {}",
			account_info.nonce, account_info.data.free
		);

		// ...or both the value and the key
		let (account_id, account_info) = iter.next_key_value().await?.expect("Must be there");
		println!(
			"Account Id: {}, Account Nonce: {}, Account Free Balance: {}",
			account_id, account_info.nonce, account_info.data.free
		);
	}

	// Example with AppKeys
	let key = "Hello World".as_bytes().to_vec();
	let app_key = DAStorage::AppKeys::fetch(&client.rpc_client, &key, None)
		.await?
		.expect("Should be there");
	println!("AppKey Owner: {}, AppKey Id: {}", app_key.owner, app_key.id);

	let mut iter = DAStorage::AppKeys::iter(client.rpc_client.clone(), client.finalized_block_hash().await?);
	for _ in 0..10 {
		// You can fetch just the value...
		let app_key = iter.next().await?.expect("Must be there");
		println!("AppKey Owner: {}, AppKey Id: {}", app_key.owner, app_key.id);

		// ...or both the value and the key
		let (key, app_key) = iter.next_key_value().await?.expect("Must be there");
		println!(
			"AppKey Key: {}, AppKey Owner: {}, AppKey Id: {}",
			String::from_utf8(key).expect("Should work"),
			app_key.owner,
			app_key.id
		);
	}

	Ok(())
}
