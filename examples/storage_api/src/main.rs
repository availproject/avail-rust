use avail::{data_availability::storage as DAStorage, system::storage as SystemStorage};
use avail_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
	// Establishing a connection
	let client = Client::new(TURING_ENDPOINT).await?;
	let rpc_client = &client.rpc_client;

	// Fetching DataAvailability::NextAppId - Storage Value
	let next_app_id = DAStorage::NextAppId::fetch(rpc_client, None)
		.await?
		.expect("Should be there");
	println!("Next App Id: {}", next_app_id.0);
	/*
		Next App Id: 484
	*/

	// Fetching DataAvailability::AppKeys - Storage Map
	let key = "Hello World".as_bytes().to_vec();
	let app_key = DAStorage::AppKeys::fetch(rpc_client, &key, None)
		.await?
		.expect("Should be there");
	println!("AppKey Owner: {}, AppKey Id: {}", app_key.owner, app_key.id);
	/*
		AppKey Owner: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY, AppKey Id: 237
	*/

	// Iterating over AppKeys
	let block_hash = client.finalized().block_hash().await?;
	let mut iter = DAStorage::AppKeys::iter(rpc_client.clone(), block_hash);
	for _ in 0..3 {
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
	/*
		AppKey Owner: 5EPRhv5CfHRWujoTMvBg58oeed8QRe4oCqHL4xtsyhY27xCo, AppKey Id: 254
		AppKey Key: My nasasasas sd sd sa ew application key, AppKey Owner: 5CqgQkrDcdg5QrtuxT3H7WszrqgrBMhdwRbmMVXQzc4VSiEg, AppKey Id: 309
		AppKey Owner: 5CocSpj62xG11MAkQMzsd7h8wRoxx1E44f8tJUUrPaWQ4opj, AppKey Id: 18
		AppKey Key: xuelan-avail-cli, AppKey Owner: 5FTZeZ1Gp81952gxXjcyDdzJmrsWjLZkFDESTHVgeKACLb66, AppKey Id: 448
		AppKey Owner: 5DDY2yzh8uCysYFAiRSTeQVwtZSKNF49CkQkyPH852xvrYKk, AppKey Id: 202
		AppKey Key: grind, AppKey Owner: 5DDSWFp79bmkDAVxMfWX5cE7LtijV7RzpmPAyTWDkeerNawd, AppKey Id: 115
	*/

	// Fetching System::Account - Storage Map
	let account_id = AccountId::from_str("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ").expect("Should work");
	let account_info = SystemStorage::Account::fetch(&client.rpc_client, &account_id, None)
		.await?
		.expect("Should be there");
	println!("Account Nonce: {}, Account Free Balance: {}", account_info.nonce, account_info.data.free);
	/*
		Account Nonce: 26, Account Free Balance: 21222260146327273433
	*/

	// Iterating over Accounts
	let mut iter = SystemStorage::Account::iter(client.rpc_client.clone(), client.finalized().block_hash().await?);
	for _ in 0..3 {
		// You can fetch just the value...
		let account_info = iter.next().await?.expect("Must be there");
		println!("Account Nonce: {}, Account Free Balance: {}", account_info.nonce, account_info.data.free);

		// ...or both the value and the key
		let (account_id, account_info) = iter.next_key_value().await?.expect("Must be there");
		println!(
			"Account Id: {}, Account Nonce: {}, Account Free Balance: {}",
			account_id, account_info.nonce, account_info.data.free
		);
	}
	/*
		Account Nonce: 0, Account Free Balance: 1000000000000000000
		Account Id: 5H6TVS3g5B1mzC3LgEKkbbhiQeyvPHn77cbcMaMcZm4RLNsn, Account Nonce: 0, Account Free Balance: 1000000000000000000
		Account Nonce: 1, Account Free Balance: 1000000000000
		Account Id: 5GiyCRSqZMoDZ23hKZTcmTyCNH19jGLmkjnLm5gTTxoNeYPz, Account Nonce: 0, Account Free Balance: 600000000000000000
		Account Nonce: 2, Account Free Balance: 74228259325442340
		Account Id: 5GuovGbSSTUDvGBcbZHXptNvptgkzMqrotN5eLXWGLajgu1h, Account Nonce: 2, Account Free Balance: 547627243140673658
	*/

	Ok(())
}
