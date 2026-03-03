use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::connect(LOCAL_ENDPOINT).await?;

	let signer = Account::new_from_str("//Bob")?;
	let account_id = signer.public_key().to_account_id();

	// Fetching account relevant information
	//
	// The easiest way to fetch account information like balance or nonce is via
	// .account() interface.
	// The following information can be fetched:
	// - .info()	- Account Nonce + Balance + Metadata
	// - .nonce()	- Account Nonce
	// - .balance()	- Account balance
	// The first input is the account id (can be string) and the second input is
	// block query mode.
	let _ = client.account().info(account_id.clone(), Default::default()).await?;
	let _ = client.account().nonce(account_id.clone(), Default::default()).await?;
	let _ = client.account().balance(account_id.clone(), Default::default()).await?;

	// If information from historical blocks are needed there are .*_at() methods
	// that can facilitate that
	let _ = client.account().info_at(account_id.clone(), 1).await?;
	let _ = client.account().nonce_at(account_id.clone(), 1).await?;
	let _ = client.account().balance_at(account_id.clone(), 1).await?;

	Ok(())
}
