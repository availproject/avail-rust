use avail::data_availability::calls::types::SubmitData;
use avail::data_availability::events::DataSubmitted;
use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	// Transaction will be signed, sent, and watched
	// If the transaction was dropped or never executed, the system will retry it
	// for 2 more times using the same nonce and app id.
	//
	// Waits for finalization to finalize the transaction.
	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);
	let res = tx
		.execute_and_watch_finalization(&account::alice(), Options::new().app_id(1))
		.await?;
	assert_eq!(res.is_successful(), Some(true));

	// Printout Transaction Details
	println!(
		"Block Hash: {:?}, Block Number: {}, Tx Hash: {:?}, Tx Index: {}",
		res.block_hash, res.block_number, res.tx_hash, res.tx_index
	);

	// Printout Transaction Events
	let events = res.events.as_ref().unwrap();
	for event in events.iter() {
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

	// Converts generic event to a specific one
	let event = events.find_first::<DataSubmitted>();
	let event = event.unwrap().unwrap();
	print!("Data Hash: {:?}, Who: {}", event.data_hash, event.who);

	// Converts generic transaction to a specific one
	let decoded = res.decode_as::<SubmitData>().await?.unwrap();
	let data = to_ascii(decoded.data.0).unwrap();
	println!("Data: {}", data);

	println!("Transaction Execute Finalization finished correctly");

	Ok(())
}
