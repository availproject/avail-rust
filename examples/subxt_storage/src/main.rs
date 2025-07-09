//! This example showcases the following actions:
//! - Fetching an decoding storage using subxt and generated metadata
//!

use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let account_id = alice().account_id();
	let storage = client.subxt_storage_client().at_latest().await?;

	let address = avail_generated::storage().system().account(&account_id);
	let account_info = storage.fetch_or_default(&address).await?;
	println!("Nonce: {}", account_info.nonce);

	let address = avail_generated::storage().system().account_iter();
	let mut stream = storage.iter(address.unvalidated()).await?;
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
