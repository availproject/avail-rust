use avail_rust::{block::to_ascii, prelude::*, utils::new_h256_from_hex};

type SubmitDataCall = avail::data_availability::calls::types::SubmitData;
type DataSubmittedEvent = avail::data_availability::events::DataSubmitted;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;
	let block_hash = new_h256_from_hex("0x94746ba186876d7407ee618d10cb6619befc59eeb173cacb00c14d1ff492fc58")?;

	let block = Block::new(&sdk.client, block_hash).await?;

	// All Transactions
	let block_transactions = block.transaction_all();
	assert_eq!(block_transactions.len(), 9, "Transaction count must be 9");

	// Printout Block Transactions
	for tx in block_transactions.iter() {
		println!(
			"Pallet Name: {:?}, Pallet Index: {}, Call Name: {:?}, Call Index: {:?}, Tx Hash: {:?}, Tx Index: {}",
			tx.pallet_name(),
			tx.pallet_index(),
			tx.call_name(),
			tx.call_index(),
			tx.tx_hash(),
			tx.tx_index()
		);

		println!(
			"Tx Signer: {:?}, App Id: {:?}, Tip: {:?}, Mortality: {:?}, Nonce: {:?}",
			tx.signer(),
			tx.app_id(),
			tx.tip(),
			tx.mortality(),
			tx.nonce(),
		);
	}

	// Convert from Block Transaction to Specific Transaction
	let da_tx = block_transactions.index(2).decode::<SubmitDataCall>();
	assert!(da_tx.is_some(), "SubmitDataCall should exist");
	let da_tx = da_tx.unwrap();
	let data = to_ascii(da_tx.data.0).unwrap();
	println!("Data: {}", data);

	// Printout all Transaction Events
	let tx_events = block_transactions.index(2).events().await;
	assert!(tx_events.is_some(), "Events should exist");
	let tx_events = tx_events.unwrap();
	assert_eq!(tx_events.len(), 7, "Event count must be 7");

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

	// Convert from Generic Transaction Event to Specific Transaction Event
	let event = tx_events.find_first::<DataSubmittedEvent>();
	assert!(
		event.as_ref().is_ok_and(|e| e.is_some()),
		"DataSubmitted event must be present"
	);
	let event = event.ok().flatten().unwrap();
	println!("Who: {}, Data Hash: {:?}", event.who, event.data_hash);

	Ok(())
}
