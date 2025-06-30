//! This example showcases the following actions:
//! - Fetching Transaction Events
//! - Fetching Block Events
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
	let submitted = tx.sign_and_submit(&alice(), Options::new().app_id(2)).await?;
	let Some(receipt) = submitted.receipt(true).await? else {
		return Err("Transaction was dropped".into());
	};

	// Find transaction related event
	let event_client = client.event_client();
	let Some(grouped_events) = event_client
		.transaction_events(receipt.tx_location.index, true, true, receipt.block_id.hash)
		.await?
	else {
		return Err("Failed to find events".into());
	};

	print_event_details(&grouped_events)?;

	// Find block related events
	let params = FetchEventsV1Params::new()
		.with_decoding(true)
		.with_encoding(true)
		.with_filter(Filter::All);
	let block_grouped_events = event_client.block_events(params, receipt.block_id.hash).await?;
	for grouped_events in block_grouped_events {
		print_event_details(&grouped_events)?;
	}

	Ok(())
}

fn print_event_details(grouped_events: &GroupedRuntimeEvents) -> Result<(), ClientError> {
	println!("Phase: {:?}", grouped_events.phase);
	for event in &grouped_events.events {
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
