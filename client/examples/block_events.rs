use avail_rust_client::prelude::*;
use avail_rust_core::rpc::EventFilter;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;
	let block = client.block(2042845);

	// Extrinsic index 0
	println!("Extrinsic 0 events: ");
	let extrinsic_events = block.events().extrinsic(0).await?;

	// Checking existence
	let is_success_present = extrinsic_events.is_extrinsic_success_present();
	let is_failed_present = extrinsic_events.is_extrinsic_failed_present();
	let multisig_executed = extrinsic_events.multisig_executed_successfully();
	let proxy_executed = extrinsic_events.proxy_executed_successfully();
	let is_deposit_present = extrinsic_events.is_present::<avail::balances::events::Deposit>();
	println!(
		"Extrinsic Success present: {}, Extrinsic Failed present: {}, Multisig Executed present: {:?}, Proxy Executed present: {:?}, Balances Deposit present: {}",
		is_success_present, is_failed_present, multisig_executed, proxy_executed, is_deposit_present
	);

	// Counting
	let success_count = extrinsic_events.count::<avail::system::events::ExtrinsicSuccess>();
	let deposit_count = extrinsic_events.count::<avail::balances::events::Deposit>();
	println!(
		"Total count: {}, Success count: {}, Deposit Count: {}",
		extrinsic_events.events.len(),
		success_count,
		deposit_count
	);
	println!("");

	// Extrinsic index 1
	println!("Extrinsic 1 events: ");
	let extrinsic_events = block.events().extrinsic(1).await?;

	// Checking existence
	let is_success_present = extrinsic_events.is_extrinsic_success_present();
	let is_failed_present = extrinsic_events.is_extrinsic_failed_present();
	let multisig_executed = extrinsic_events.multisig_executed_successfully();
	let proxy_executed = extrinsic_events.proxy_executed_successfully();
	let is_deposit_present = extrinsic_events.is_present::<avail::balances::events::Deposit>();
	println!(
		"Extrinsic Success present: {}, Extrinsic Failed present: {}, Multisig Executed present: {:?}, Proxy Executed present: {:?}, Balances Deposit present: {}",
		is_success_present, is_failed_present, multisig_executed, proxy_executed, is_deposit_present
	);

	// Counting
	let success_count = extrinsic_events.count::<avail::system::events::ExtrinsicSuccess>();
	let deposit_count = extrinsic_events.count::<avail::balances::events::Deposit>();
	println!(
		"Total count: {}, Success count: {}, Deposit Count: {}",
		extrinsic_events.events.len(),
		success_count,
		deposit_count
	);
	println!("");

	// Extrinsic Events
	let extrinsic_events = block.events().extrinsic(1).await?;
	let first = extrinsic_events
		.first::<avail::balances::events::Withdraw>()
		.expect("Should be decodable");
	let last = extrinsic_events
		.first::<avail::system::events::ExtrinsicSuccess>()
		.expect("Should be decodable");
	let all = extrinsic_events.all::<avail::balances::events::Deposit>()?;

	println!(
		"Withdraw Amount: {}, Extrinsic Weight: {}, Deposits Count: {}",
		first.amount,
		last.dispatch_info.weight.ref_time,
		all.len()
	);

	// 1. Decoding ExtrinsicSuccess event
	let extrinsic_0_events = block.events().extrinsic(0).await?;
	let extrinsic_1_events = block.events().extrinsic(1).await?;
	let event_0 = extrinsic_0_events
		.first::<avail::system::events::ExtrinsicSuccess>()
		.expect("Should be decodable");
	let event_1 = extrinsic_1_events
		.first::<avail::system::events::ExtrinsicSuccess>()
		.expect("Should be decodable");
	println!(
		"1. Timestamp::Set Weight: {}, DataAvailability::SubmitData Weight: {}",
		event_0.dispatch_info.weight.ref_time, event_1.dispatch_info.weight.ref_time,
	);

	// 2. Decoding Balances::Deposit Event
	println!("Displaying all Balances::Deposit events");
	let extrinsic_events = block.events().extrinsic(1).await?;
	let deposits = extrinsic_events.all::<avail::balances::events::Deposit>()?;
	for deposit in deposits {
		println!("2. Account ID: {}, Amount: {}", deposit.who, deposit.amount)
	}
	println!("");

	// 3. Decoding DataAvailability::DataSubmitted Event
	println!("Displaying DataAvailability::DataSubmitted event");
	let extrinsic_events = block.events().extrinsic(1).await?;
	let event = extrinsic_events
		.first::<avail::data_availability::events::DataSubmitted>()
		.expect("Should be decodable");
	println!("3. Who: {}, Data Hash: {:?}", event.who, event.data_hash);
	println!("");

	// 4. Decoding TransactionPayment::TransactionFeePaid Event
	println!("Displaying TransactionPayment::TransactionFeePaid event");
	let extrinsic_events = block.events().extrinsic(1).await?;
	let event = extrinsic_events
		.first::<avail::transaction_payment::events::TransactionFeePaid>()
		.expect("Should be decodable");
	println!("4. Who: {}, Actual Fee: {}, Tip: {}", event.who, event.actual_fee, event.tip);
	println!("");

	// Block System Events
	println!("Displaying block system event");
	let system_events = block.events().system().await?;
	let event = system_events
		.first::<avail::treasury::events::UpdatedInactive>()
		.expect("Should be decodable");
	println!("5. Reactivated: {}, Deactivated: {}", event.reactivated, event.deactivated);
	println!("");

	// All Events (both Extrinsic and Block)
	println!("Iterating over all extrinsic index 0 events");
	let events = block.events().extrinsic(0).await?;
	for event in events.events {
		println!(
			"	Index: {}, Pallet ID: {}, Variant ID: {}, Data Length: {}, Phase: {:?}",
			event.index,
			event.pallet_id,
			event.variant_id,
			event.data.len(),
			event.phase,
		);

		if (event.pallet_id, event.variant_id) == avail::balances::events::Withdraw::HEADER_INDEX {
			let withdraw = avail::balances::events::Withdraw::from_event(event.data).expect("Should be decodable");
			println!("	    6. Who: {}, amount: {}", withdraw.who, withdraw.amount);
		}
	}

	println!("Iterating over all block system events");
	let events = block.events().system().await?;
	for event in events.events {
		println!(
			"	Index: {}, Pallet ID: {}, Variant ID: {}, Data Length: {}, Phase: {:?}",
			event.index,
			event.pallet_id,
			event.variant_id,
			event.data.len(),
			event.phase,
		);

		if (event.pallet_id, event.variant_id) == avail::balances::events::Withdraw::HEADER_INDEX {
			let withdraw = avail::balances::events::Withdraw::from_event(event.data).expect("Should be decodable");
			println!("	    6. Who: {}, amount: {}", withdraw.who, withdraw.amount);
		}
	}

	println!("Iterating over all events (both extrinsic and system events)");
	let events = block.events().all(EventFilter::All).await?;
	for event in events {
		println!(
			"	Index: {}, Pallet ID: {}, Variant ID: {}, Data Length: {}, Phase: {:?}",
			event.index,
			event.pallet_id,
			event.variant_id,
			event.data.len(),
			event.phase,
		);

		if (event.pallet_id, event.variant_id) == avail::balances::events::Withdraw::HEADER_INDEX {
			let withdraw = avail::balances::events::Withdraw::from_event(event.data).expect("Should be decodable");
			println!("	    6. Who: {}, amount: {}", withdraw.who, withdraw.amount);
		}
	}

	Ok(())
}

/*
	Expected Output:

	Extrinsic 0 events:
	Extrinsic Success present: true, Extrinsic Failed present: false, Multisig Executed present: None, Proxy Executed present: None, Balances Deposit present: false
	Total count: 1, Success count: 1, Deposit Count: 0

	Extrinsic 1 events:
	Extrinsic Success present: true, Extrinsic Failed present: false, Multisig Executed present: None, Proxy Executed present: None, Balances Deposit present: true
	Total count: 7, Success count: 1, Deposit Count: 3

	Withdraw Amount: 124711139352751361, Extrinsic Weight: 13057471500, Deposits Count: 3
	1. Timestamp::Set Weight: 12606212000, DataAvailability::SubmitData Weight: 13057471500
	Displaying all Balances::Deposit events
	2. Account ID: 5EZZm8AKzZw8ti9PSmTZdXCgNEeaE3vs5sNxqkQ6u5NhG8kT, Amount: 0
	2. Account ID: 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z, Amount: 99768911482201088
	2. Account ID: 5Ew2zpT4iT7fRLqD81fzq7rGViVj4MSLKMJn6tZdadbQLy8B, Amount: 24942227870550273

	Displaying DataAvailability::DataSubmitted event
	3. Who: 5EZZm8AKzZw8ti9PSmTZdXCgNEeaE3vs5sNxqkQ6u5NhG8kT, Data Hash: 0x14e3128c0c0f5840c1594420546b1dbd2ed60ac6f8f9095a06db7ad1a19032bf

	Displaying TransactionPayment::TransactionFeePaid event
	4. Who: 5EZZm8AKzZw8ti9PSmTZdXCgNEeaE3vs5sNxqkQ6u5NhG8kT, Actual Fee: 124711139352751361, Tip: 0

	Displaying block system event
	5. Reactivated: 292332715967391734000945630, Deactivated: 292332716069790243286966271

	Iterating over all extrinsic index 0 events.
		Index: 1, Pallet ID: 0, Variant ID: 0, Data Length: 30, Phase: ApplyExtrinsic(0)
	Iterating over all block system events.
		Index: 0, Pallet ID: 18, Variant ID: 8, Data Length: 68, Phase: Initialization
	Iterating over all events (both extrinsic and system events).
		Index: 0, Pallet ID: 18, Variant ID: 8, Data Length: 68, Phase: Initialization
		Index: 1, Pallet ID: 0, Variant ID: 0, Data Length: 30, Phase: ApplyExtrinsic(0)
		Index: 2, Pallet ID: 6, Variant ID: 8, Data Length: 100, Phase: ApplyExtrinsic(1)
			6. Who: 5EZZm8AKzZw8ti9PSmTZdXCgNEeaE3vs5sNxqkQ6u5NhG8kT, amount: 124711139352751361
		Index: 3, Pallet ID: 29, Variant ID: 1, Data Length: 132, Phase: ApplyExtrinsic(1)
		Index: 4, Pallet ID: 6, Variant ID: 7, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 5, Pallet ID: 6, Variant ID: 7, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 6, Pallet ID: 6, Variant ID: 7, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 7, Pallet ID: 7, Variant ID: 0, Data Length: 132, Phase: ApplyExtrinsic(1)
		Index: 8, Pallet ID: 0, Variant ID: 0, Data Length: 36, Phase: ApplyExtrinsic(1)
		Index: 9, Pallet ID: 0, Variant ID: 0, Data Length: 28, Phase: ApplyExtrinsic(2)
*/
