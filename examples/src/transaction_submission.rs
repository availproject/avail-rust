//! This example showcases the following actions:
//! - Fetching Account Balance
//! - Fetching Account Nonce
//! - Fetching Account Info (contains account balance and nonce)
//!

use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let client = Client::new(LOCAL_ENDPOINT).await?;
	let signer = alice();

	let submittable_tx = client
		.tx()
		.data_availability()
		.create_application_key(vec![0, 1, 2, 3, 4, 5]);
	let submitted_tx = submittable_tx
		.sign_and_submit(&signer, Options::new().app_id(0))
		.await?;
	println!(
		"Tx Hash: {:?}, Account Address: {}, Used Options: {:?}, Used Additional: {:?}",
		submitted_tx.tx_hash, submitted_tx.account_id, submitted_tx.options, submitted_tx.additional
	);

	let receipt = submitted_tx.receipt(true).await?;
	let Some(receipt) = receipt else {
		println!("Transaction was not included in any block.");
		return Ok(());
	};
	println!(
		"Block Hash: {:?}, Block Height: {}, Tx Hash: {:?}, Tx Index: {}",
		receipt.block_id.hash, receipt.block_id.height, receipt.tx_location.hash, receipt.tx_location.index
	);

	let block_state = receipt.block_state().await?;
	match block_state {
		BlockState::Included => println!("Block is included but not finalized"),
		BlockState::Finalized => println!("Block is finalized"),
		BlockState::Discarded => println!("Block is discarded"),
		BlockState::DoesNotExist => println!("Block does not exist"),
	}

	let (tx_index, block_hash) = (receipt.tx_location.index, receipt.block_id.hash);
	let events_client = client.events_client();
	let events = events_client.transaction_events(tx_index, block_hash).await?;
	for event in events {
		let myb = RuntimeEvent::try_from(&event);
		dbg!(myb);
	}

	Ok(())
}
