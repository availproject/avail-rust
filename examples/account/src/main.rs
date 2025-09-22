//! This example showcases the following actions:
//! - Creating Keypair from mnemonic seed
//! - Creating Account Id from SS58 address or Keypair
//! - Displaying the SS58 Address of a Account Id
//! - Fetching Account Balance, Nonce and Info
//!
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	keypair_examples()?;
	account_id_examples()?;
	account_information().await?;

	Ok(())
}

fn keypair_examples() -> Result<(), ClientError> {
	// Creating Keypair from mnemonic seed
	let development = AccountId::from_str("5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV")?;
	let keypair = Keypair::from_str("bottom drive obey lake curtain smoke basket hold race lonely fit walk")?;
	assert_eq!(keypair.account_id(), development);

	// Creating Keypair from mnemonic seed with hard derivation
	let alice = AccountId::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")?;
	let keypair = Keypair::from_str("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice")?;
	assert_eq!(keypair.account_id(), alice);

	let alice_stash = AccountId::from_str("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY")?;
	let keypair =
		Keypair::from_str("bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice//stash")?;
	assert_eq!(keypair.account_id(), alice_stash);

	// Creating Keypair from Raw Seed
	let account_id = AccountId::from_str("5HVSLMgPW5ZNi8755scgY7dnCK39ZYEhYnNFUpggqog2sN76")?;
	let keypair = Keypair::from_str("0x2246b68b2f9050f1eb38e44f1f0abd065b5694cc88dd44695af19b1e5fff344f")?;
	assert_eq!(keypair.account_id(), account_id);

	Ok(())
}

fn account_id_examples() -> Result<(), ClientError> {
	// Account Id from String
	let alice = AccountId::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")?;
	assert_eq!(alice.to_string(), "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");

	// Account Id from Keypair
	let keypair = Keypair::from_str("//Alice")?;
	assert_eq!(keypair.account_id(), alice);

	// Account Id from Raw
	let raw = AccountId::from_slice(&vec![
		212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227,
		154, 86, 132, 231, 165, 109, 162, 125,
	])?;
	assert_eq!(raw, alice);

	// Account Id to SS58 Address
	assert_eq!(alice.to_string(), "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
	assert_eq!(std::format!("{}", alice), "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");

	Ok(())
}

async fn account_information() -> Result<(), ClientError> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Account Balance
	let account_id = AccountId::from_str("5DUhCbe3dcrGEFkUn7fjSvd1DpCqUfg6X9tMmKCwLpSfHKCS")?;
	let finalized_block_hash = client.finalized().block_hash().await?;

	let _balance = client.rpc().balance(&account_id, finalized_block_hash).await?;
	let _balance = client.best().block_balance(&account_id).await?;
	let balance = client.finalized().block_balance(&account_id).await?;
	println!("Free: {}, Reserved: {}, Frozen: {}", balance.free, balance.reserved, balance.frozen);

	// Account Nonce
	let account_id = AccountId::from_str("5HN2ZfzS6i87nxxv7Rbugob4KaYGD2B4xNq3ECkHfCkDZrTK")?;
	let _nonce = client.rpc().nonce(&account_id).await?;
	let _nonce = client.rpc().block_nonce(&account_id, finalized_block_hash).await?;
	let _nonce = client.best().block_nonce(&account_id).await?;
	let nonce = client.finalized().block_nonce(&account_id).await?;
	println!("Address: {}, Nonce: {}", account_id, nonce);

	// Account Info
	let account_id = AccountId::from_str("5Hn8x2fstQmcqLg4C8pEiLWdAJhGaRv8jfYRUrnHeiMALvAX")?;
	let _info = client.rpc().account_info(&account_id, finalized_block_hash).await?;
	let _info = client.best().block_account_info(&account_id).await?;
	let info = client.finalized().block_account_info(&account_id).await?;
	println!("Nonce: {}, Free Balance: {}", info.nonce, info.data.free);

	Ok(())
}
