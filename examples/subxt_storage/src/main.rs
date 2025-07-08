use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let account_id = alice().account_id();
	let finalized_block_hash = client.finalized_block_hash().await?;
	let storage = client.storage_client();
	let address = avail::system::storage::account(&account_id);
	let account_info = storage.fetch_or_default(&address, finalized_block_hash).await?;
	println!("Nonce: {}", account_info.nonce);

	let address = avail::system::storage::account_iter();
	let mut stream = storage.iter(address.unvalidated(), finalized_block_hash).await?;
	while let Some(Ok(info)) = stream.next().await {
		if info.key_bytes.len() <= 32 {
			continue;
		}
		let account_id_raw = info.key_bytes.last_chunk::<32>().expect("Checked");
		let account_id = AccountId::from(*account_id_raw);
		println!("Account Id: {}", account_id);
		println!("Nonce: {}", info.value.nonce);
	}
	Ok(())
}
