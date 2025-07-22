//! This example showcases the following actions:
//! - Fetching block and transaction events via event client
//! - Decoding block and transaction events
//! - Fetching and Decoding events from historical blocks
//!

use avail::{
	data_availability::events::DataSubmitted as DataSubmittedEvent,
	system::events::ExtrinsicSuccess as ExtrinsicSuccessEvent,
};
use avail_rust_client::{
	avail_rust_core::{
		FetchEventsV1Options,
		rpc::system::fetch_events_v1_types::{Filter, GroupedRuntimeEvents, RuntimeEvent},
	},
	prelude::*,
};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	// Submit transaction
	let tx = client.tx().data_availability().submit_data(vec![b'a']);
	let submitted = tx.sign_and_submit(&alice(), Options::new(Some(2))).await?;
	let Some(receipt) = submitted.receipt(true).await? else {
		return Err("Transaction was dropped".into());
	};

	// Fetching transaction events directly via receipt
	let event_group = receipt.tx_events().await?;
	print_events(&event_group)?;

	// Find transaction related event
	let event_client = client.event_client();
	let Some(event_group) = event_client
		.transaction_events(receipt.tx_loc.index, true, true, receipt.block_loc.hash)
		.await?
	else {
		return Err("Failed to find events".into());
	};

	print_events(&event_group)?;

	// Find block related events
	let params = FetchEventsV1Options::new(Some(Filter::All), Some(true), Some(true));
	let block_event_group = event_client.block_events(receipt.block_loc.hash, Some(params)).await?;
	for event_group in block_event_group {
		print_grouped_events(&event_group)?;
	}

	// Fetching historical block events
	historical_block_events(&client, receipt.block_loc.hash, receipt.tx_loc.index).await?;

	Ok(())
}

fn print_grouped_events(event_group: &GroupedRuntimeEvents) -> Result<(), ClientError> {
	println!("Phase: {:?}", event_group.phase);
	print_events(&event_group.events)?;

	Ok(())
}

fn print_events(events: &Vec<RuntimeEvent>) -> Result<(), ClientError> {
	for event in events {
		println!(
			"Event Index: {}, Pallet Id: {}, Variant id: {}",
			event.index, event.emitted_index.0, event.emitted_index.1
		);
		let Some(encoded) = &event.encoded else {
			return Err("Event was supposed to be encoded".into());
		};
		println!("Event (hex and string) encoded value: 0x{}", encoded);

		if let Some(decoded) = &event.decoded {
			println!("Event (hex and string) decoded value: 0x{}", decoded);
		} else {
			println!("The event was not decoded");
		}

		let event = const_hex::decode(encoded)?;
		if let Some(e) = ExtrinsicSuccessEvent::decode_event(&event) {
			println!("Weight: {:?}", e.dispatch_info.weight)
		}
		if let Some(e) = DataSubmittedEvent::decode_event(&event) {
			println!("Who: {}, Data Hash: {:?}", e.who, e.data_hash)
		}
	}

	Ok(())
}

async fn historical_block_events(client: &Client, at: H256, tx_index: u32) -> Result<(), ClientError> {
	use subxt_core::events::Phase;
	let event_client = client.event_client();
	let events = event_client.historical_block_events(at).await?;
	for event in events {
		match &event.phase {
			Phase::ApplyExtrinsic(x) if *x == tx_index => (),
			_ => continue,
		}

		println!(
			"Pallet id: {}, Variant id: {}, Event Data: {:?}",
			event.pallet_index(),
			event.variant_index(),
			event.event_data(),
		);

		if let Some(e) = ExtrinsicSuccessEvent::decode_event(event.event_bytes()) {
			println!("Weight: {:?}", e.dispatch_info.weight)
		}
		if let Some(e) = DataSubmittedEvent::decode_event(event.event_bytes()) {
			println!("Who: {}, Data Hash: {:?}", e.who, e.data_hash)
		}
	}

	Ok(())
}
