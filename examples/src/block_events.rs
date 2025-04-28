use avail_rust::prelude::*;

type TransferEvent = avail::balances::events::Transfer;
type AppKeyCreatedEvent = avail::data_availability::events::ApplicationKeyCreated;
type DataSubmittedEvent = avail::data_availability::events::DataSubmitted;
type SuccessEvent = avail::system::events::ExtrinsicSuccess;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;
	let block_hash = H256::from_hex("0x94746ba186876d7407ee618d10cb6619befc59eeb173cacb00c14d1ff492fc58")?;

	let block = Block::new(&sdk.client, block_hash).await?;

	// All Block Events
	let block_events = block.events().await;
	assert!(block_events.is_some(), "Events must been present");
	let block_events = block_events.unwrap();
	assert_eq!(block_events.len(), 53, "Block event count must be 53");

	// Printout All Block Events
	for event in block_events.iter() {
		let tx_index = match event.phase() {
			subxt::events::Phase::ApplyExtrinsic(x) => Some(x),
			_ => None,
		};

		println!(
			"Pallet Name: {}, Pallet Index: {}, Event Name: {}, Event Index: {}, Event Position: {}, Tx Index: {:?}",
			event.pallet_name(),
			event.pallet_index(),
			event.variant_name(),
			event.variant_index(),
			event.index(),
			tx_index,
		);
	}

	// Convert from Block Transaction Event to Specific Transaction Event
	let events = block_events.find::<TransferEvent>();
	assert_eq!(events.len(), 2, "Transfer event count must be 2");

	for event in events {
		println!("From: {}, To: {}, Amount: {}", event.from, event.to, event.amount)
	}

	// Convert from Block Transaction Event to Specific ApplicationKeyCreated Event
	let event = block_events.find_first::<AppKeyCreatedEvent>();
	assert!(event.as_ref().is_some_and(|x| x.is_some()), "AppKeyCreatedEvent");

	let event = event.unwrap().unwrap();
	let key: String = to_ascii(event.key.0).unwrap();
	println!("Owner: {}, App Id: {}, Key: {}", event.owner, event.id.0, key);

	// Check
	assert_eq!(
		block_events.find::<DataSubmittedEvent>().len(),
		4,
		"DataSubmitted event count must be 4"
	);
	assert_eq!(
		block_events.find::<AppKeyCreatedEvent>().len(),
		1,
		"AppKeyCreated event count must be 1"
	);

	// Events for Specific Transaction
	let tx_index = 0u32;
	let tx_events = block.tx_events(tx_index).await;
	assert!(tx_events.is_some(), "Transaction Events must be present");
	let tx_events = tx_events.unwrap();
	assert_eq!(tx_events.len(), 1, "Transaction event count must be 1");

	// Printout All Tx Events
	for event in tx_events.iter() {
		let tx_index = match event.phase() {
			subxt::events::Phase::ApplyExtrinsic(x) => Some(x),
			_ => None,
		};

		println!(
			"Pallet Name: {}, Pallet Index: {}, Event Name: {}, Event Index: {}, Event Position: {}, Tx Index: {:?}",
			event.pallet_name(),
			event.pallet_index(),
			event.variant_name(),
			event.variant_index(),
			event.index(),
			tx_index,
		);
	}

	// Convert from Block Transaction Event to Specific Transaction Event
	let event = tx_events.find_first::<SuccessEvent>();
	assert!(event.as_ref().is_some_and(|x| x.is_some()), "SuccessEvent");

	let event = event.unwrap().unwrap();
	println!("Weight {:?}", event.dispatch_info.weight);

	Ok(())
}
