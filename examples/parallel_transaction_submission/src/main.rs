//! This example showcases the following actions:
//! - Submitting multiple transactions at the same time
//!

use avail_rust::prelude::*;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_logging();
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let mut futures: Vec<JoinHandle<Result<(), ClientError>>> = Vec::new();
	for signer in [alice(), bob(), charlie(), dave()] {
		let s = client.clone();
		futures.push(tokio::spawn(async move { task(s, signer).await }));
	}

	for fut in futures {
		fut.await.unwrap()?;
	}

	Ok(())
}

async fn task(client: Client, account: Keypair) -> Result<(), ClientError> {
	// Transaction Submission
	let message = String::from("It works").as_bytes().to_vec();
	let tx = client.tx().data_availability().submit_data(message);
	let st = tx.sign_and_submit(&account, Options::new()).await?;

	// Fetching Transaction Receipt
	let Some(receipt) = st.receipt(false).await? else {
		return Err("Transaction got dropped. This should never happen in a local network.".into());
	};

	// Fetching Block State
	let block_state = receipt.block_state().await?;
	println!("Block State: {:?}", block_state);

	Ok(())
}
