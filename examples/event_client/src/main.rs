//! This example showcases the following actions:
//! - Fetching block and transaction events via event client
//! - Decoding block and transaction events
//!

use avail_rust_client::{
	avail_rust_core::{
		FetchEventsV1Params,
		avail::RuntimeEvent,
		rpc::system::fetch_events_v1_types::{Filter, GroupedRuntimeEvents},
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
	print_event_details(&event_group)?;

	// Find transaction related event
	let event_client = client.event_client();
	let Some(event_group) = event_client
		.transaction_events(receipt.tx_loc.index, true, true, receipt.block_loc.hash)
		.await?
	else {
		return Err("Failed to find events".into());
	};

	print_event_details(&event_group)?;

	// Find block related events
	let params = FetchEventsV1Params::new(Some(Filter::All), Some(true), Some(true));
	let block_event_group = event_client.block_events(params, receipt.block_loc.hash).await?;
	for event_group in block_event_group {
		print_event_details(&event_group)?;
	}

	Ok(())
}

fn print_event_details(event_group: &GroupedRuntimeEvents) -> Result<(), ClientError> {
	println!("Phase: {:?}", event_group.phase);
	for event in &event_group.events {
		println!(
			"Event pallet id: {}, Event event id: {}",
			event.emitted_index.0, event.emitted_index.1
		);
		let Some(encoded) = &event.encoded else {
			return Err("Event was supposed to be encoded".into());
		};
		println!("Event (hex and string) encoded value: {}", encoded);

		if let Some(decoded) = &event.decoded {
			println!("Event (hex and string) decoded value: {}", decoded);
		} else {
			println!("The event was not decoded");
		}

		// Decoding the event
		let Ok(encoded) = hex::decode(encoded.trim_start_matches("0x")) else {
			return Err("Failed to decode encoded event".into());
		};
		let Ok(runtime_event) = RuntimeEvent::try_from(&encoded) else {
			println!("Could note decode the runtime event");
			continue;
		};
		dbg!(runtime_event);
	}

	Ok(())
}
