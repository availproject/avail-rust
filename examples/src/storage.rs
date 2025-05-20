use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let account_id = alice().account_id();
	let finalized_block_hash = client.finalized_block_hash().await?;
	let storage = client.storage_client();
	let address = avail::system::storage::account(&account_id);
	let account_info = storage.fetch_or_default(&address, finalized_block_hash).await?;
	println!("Nonce: {}", account_info.nonce);

	/* 	let address = avail::system::storage::account_iter();
	let account_info = storage.fetch_or_default(&address, finalized_block_hash).await?;
	println!("Nonce: {}", account_info.nonce); */

	Ok(())
}
