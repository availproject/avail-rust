//! This example showcases the following actions:
//! - Transaction Creation
//! - Transaction Submission
//! - Fetching Transaction Receipt
//! - Fetching Block State
//! - Fetching and displaying Transaction Events
//! - Fetching Block Transaction
//!

use avail::data_availability::{events::DataSubmitted, tx::SubmitData};
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;
	let signer = alice();

	// Transaction Creation
	let submittable_tx = client.tx().data_availability().submit_data(vec![0, 1, 2, 3, 4, 5]);

	// Transaction Submission
	let submitted_tx = submittable_tx.sign_and_submit(&signer, Options::new(Some(2))).await?;
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
		receipt.block_loc.hash, receipt.block_loc.height, receipt.tx_loc.hash, receipt.tx_loc.index
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
	let event_group = receipt.tx_events().await?;
	for event in event_group.events {
		println!(
			"Pallet Index: {}, Variant index: {}",
			event.emitted_index.0, event.emitted_index.1,
		);

		let encoded_event = hex::decode(event.encoded.expect("Must be there")).expect("Must be ok");
		if let Some(event) = DataSubmitted::decode_event(&encoded_event) {
			println!("Who: {}, Data Hash: {}", event.who, event.data_hash);
		}
	}

	// Fetching the same transaction from the block
	let block_client = client.block_client();
	let block_tx = block_client
		.block_transaction(receipt.block_loc.into(), receipt.tx_loc.into(), None, None)
		.await?
		.expect("Must be there");
	let call = SubmitData::decode_hex_call(&block_tx.encoded.expect("Must be there")).expect("Must be decodable");
	println!("Call Data: {:?}", call.data);

	Ok(())
}
