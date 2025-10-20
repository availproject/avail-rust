use avail_rust_client::{block::EventsOpts, prelude::*};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;
	let block = client.block(2042845);

	// Extrinsic index 0
	println!("Extrinsic 0 events: ");
	let extrinsic_events = block
		.events()
		.extrinsic(0)
		.await?
		.expect("Events should exist for first extrinsic");

	// Checking existence
	let is_success_present = extrinsic_events.is_extrinsic_success_present();
	let is_failed_present = extrinsic_events.is_extrinsic_failed_present();
	let multisig_executed = extrinsic_events.multisig_executed_successfully();
	let proxy_executed = extrinsic_events.proxy_executed_successfully();
	let is_deposit_present = extrinsic_events.is_present::<avail::balances::events::Deposit>();
	println!(
		"1. Extrinsic Success present: {}, Extrinsic Failed present: {}, Multisig Executed present: {:?}, Proxy Executed present: {:?}, Balances Deposit present: {}",
		is_success_present, is_failed_present, multisig_executed, proxy_executed, is_deposit_present
	);

	// Counting
	let success_count = extrinsic_events.count::<avail::system::events::ExtrinsicSuccess>();
	let deposit_count = extrinsic_events.count::<avail::balances::events::Deposit>();
	println!(
		"Total count: {}, Success count: {}, Withdraw Count: {}",
		extrinsic_events.events.len(),
		success_count,
		deposit_count
	);
	println!("");

	// Extrinsic index 1
	println!("Extrinsic 1 events: ");
	let extrinsic_events = block
		.events()
		.extrinsic(1)
		.await?
		.expect("Events should exist for first extrinsic");

	// Checking existence
	let is_success_present = extrinsic_events.is_extrinsic_success_present();
	let is_failed_present = extrinsic_events.is_extrinsic_failed_present();
	let multisig_executed = extrinsic_events.multisig_executed_successfully();
	let proxy_executed = extrinsic_events.proxy_executed_successfully();
	let is_deposit_present = extrinsic_events.is_present::<avail::balances::events::Deposit>();
	println!(
		"1. Extrinsic Success present: {}, Extrinsic Failed present: {}, Multisig Executed present: {:?}, Proxy Executed present: {:?}, Balances Deposit present: {}",
		is_success_present, is_failed_present, multisig_executed, proxy_executed, is_deposit_present
	);

	// Counting
	let success_count = extrinsic_events.count::<avail::system::events::ExtrinsicSuccess>();
	let deposit_count = extrinsic_events.count::<avail::balances::events::Deposit>();
	println!(
		"Total count: {}, Success count: {}, Withdraw Count: {}",
		extrinsic_events.events.len(),
		success_count,
		deposit_count
	);
	println!("");

	// Extrinsic Events
	// All
	println!("Displaying all Balances::Deposit events");
	let deposits = extrinsic_events.all::<avail::balances::events::Deposit>()?;
	for deposit in deposits {
		println!("2. Account ID: {}, Amount: {}", deposit.who, deposit.amount)
	}
	println!("");

	// First
	println!("Displaying first DataAvailability::DataSubmitted event");
	let event = extrinsic_events
		.first::<avail::data_availability::events::DataSubmitted>()
		.expect("Should be decodable");
	println!("3. Who: {}, Data Hash: {:?}", event.who, event.data_hash);
	println!("");

	// Last
	println!("Displaying last TransactionPayment::TransactionFeePaid event");
	let event = extrinsic_events
		.last::<avail::transaction_payment::events::TransactionFeePaid>()
		.expect("Should be decodable");
	println!("4. Who: {}, Actual Fee: {}, Tip: {}", event.who, event.actual_fee, event.tip);
	println!("");

	// Block Events
	println!("Displaying block event");
	let block_events = block.events().block().await?;
	let event = block_events
		.first::<avail::treasury::events::UpdatedInactive>()
		.expect("Should be decodable");
	println!("5. Reactivated: {}, Deactivated: {}", event.reactivated, event.deactivated);
	println!("");

	// All Events (both Extrinsic and Block)
	println!("Displaying all events");
	let all_phase_events = block.events().all(EventsOpts::new().enable_encoding(true)).await?;
	for phase_events in all_phase_events {
		println!("Phase: {:?}", phase_events.phase);
		for event in phase_events.events {
			println!(
				"	Index: {}, Pallet ID: {}, Variant ID: {}, Data Length: {}",
				event.index,
				event.pallet_id,
				event.variant_id,
				event.encoded_data.as_ref().map(|x| x.len()).unwrap_or(0)
			);

			if (event.pallet_id, event.variant_id) == avail::balances::events::Withdraw::HEADER_INDEX {
				let withdraw =
					avail::balances::events::Withdraw::from_event(event.encoded_data.unwrap_or_else(|| String::new()))
						.expect("Should be decodable");
				println!("	6. Who: {}, amount: {}", withdraw.who, withdraw.amount);
			}
		}
	}

	Ok(())
}
