//! This example showcases the following actions:
//! - Transaction Creation
//! - Transaction Submission
//! - Fetching Transaction Receipt
//! - Fetching Block State
//! - Fetching and displaying Transaction Events
//!

use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;
	let signer = alice();

	// Transaction Creation
	let submittable_tx = client.tx().data_availability().submit_data(vec![0, 1, 2, 3, 4, 5]);

	// Transaction Submission
	let submitted_tx = submittable_tx
		.sign_and_submit(&signer, Options::new().app_id(2))
		.await?;
	println!(
		"Tx Hash: {:?}, Account Address: {}, Used Options: {:?}, Used Additional: {:?}",
		submitted_tx.tx_hash, submitted_tx.account_id, submitted_tx.options, submitted_tx.additional
	);

	// Fetching Transaction Receipt
	let receipt = submitted_tx.receipt(false).await?;
	let Some(receipt) = receipt else {
		return Err("Transaction got dropped. This should never happen in a local network.".into());
	};
	println!(
		"Block Hash: {:?}, Block Height: {}, Tx Hash: {:?}, Tx Index: {}",
		receipt.block_id.hash, receipt.block_id.height, receipt.tx_location.hash, receipt.tx_location.index
	);

	// Fetching Block State
	let block_state = receipt.block_state().await?;
	match block_state {
		BlockState::Included => println!("Block is included but not finalized"),
		BlockState::Finalized => println!("Block is finalized"),
		BlockState::Discarded => println!("Block is discarded"),
		BlockState::DoesNotExist => println!("Block does not exist"),
	}

	// Fetching and displaying Transaction Events
	let (tx_index, block_hash) = (receipt.tx_location.index, receipt.block_id.hash);
	let events_client = client.event_client();
	let events = events_client.transaction_events(tx_index, block_hash).await?;
	for event in events {
		println!(
			"Pallet Index: {}, Variant index: {}",
			event.pallet_index(),
			event.variant_index()
		);
		if let Ok(event) = RuntimeEvent::try_from(&event) {
			dbg!(event);
		}
	}

	Ok(())
}
