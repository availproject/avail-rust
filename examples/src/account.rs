//! This example showcases the following actions:
//! - Fetching Account Balance
//! - Fetching Account Nonce
//! - Fetching Account Info (contains account balance and nonce)
//!

use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	// Account Balance
	let account_id = AccountId::from_str("5DUhCbe3dcrGEFkUn7fjSvd1DpCqUfg6X9tMmKCwLpSfHKCS")?;
	let finalized_block_hash = client.finalized_block_hash().await?;

	let balance = client.balance(&account_id, finalized_block_hash).await?;
	println!(
		"Free: {}, Reserved: {}, Frozen: {}",
		balance.free, balance.reserved, balance.frozen
	);
	let balance = client.best_block_balance(&account_id).await?;
	println!(
		"Free: {}, Reserved: {}, Frozen: {}",
		balance.free, balance.reserved, balance.frozen
	);
	let balance = client.finalized_block_balance(&account_id).await?;
	println!(
		"Free: {}, Reserved: {}, Frozen: {}",
		balance.free, balance.reserved, balance.frozen
	);

	// Account Nonce
	let account_id = AccountId::from_str("5HN2ZfzS6i87nxxv7Rbugob4KaYGD2B4xNq3ECkHfCkDZrTK")?;
	let nonce = client.nonce(&account_id).await?;
	println!("Address: {}, Nonce: {}", account_id, nonce);
	let nonce = client.block_nonce(&account_id, finalized_block_hash).await?;
	println!("Address: {}, Nonce: {}", account_id, nonce);
	let nonce = client.best_block_nonce(&account_id).await?;
	println!("Address: {}, Nonce: {}", account_id, nonce);
	let nonce = client.finalized_block_nonce(&account_id).await?;
	println!("Address: {}, Nonce: {}", account_id, nonce);

	// Account Info
	let account_id = AccountId::from_str("5Hn8x2fstQmcqLg4C8pEiLWdAJhGaRv8jfYRUrnHeiMALvAX")?;
	let info = client.account_info(&account_id, finalized_block_hash).await?;
	println!("Nonce: {}, Free Balance: {}", info.nonce, info.data.free);
	let info = client.best_block_account_info(&account_id).await?;
	println!("Nonce: {}, Free Balance: {}", info.nonce, info.data.free);
	let info = client.finalized_block_account_info(&account_id).await?;
	println!("Nonce: {}, Free Balance: {}", info.nonce, info.data.free);

	Ok(())
}
