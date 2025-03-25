use avail::data_availability::{calls::types::SubmitData, events::DataSubmitted};
use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	// Transaction will be signed, and sent.
	//
	// There is no guarantee that the transaction was executed at all. It might have been
	// dropped or discarded for various reasons. The caller is responsible for querying future
	// blocks in order to determine the execution status of that transaction.
	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);
	let tx_hash = tx.execute(&account::alice(), Options::new().app_id(1)).await?;
	println!("Tx Hash: {:?}", tx_hash);

	// Checking if the transaction was included
	//
	// It's not necessary to use the builtin watcher. A custom watcher
	// might yield better results in some cases.
	let mut watcher = Watcher::new(sdk.client.clone(), tx_hash);
	watcher.set_options(|opt: &mut WatcherOptions| opt.wait_for = WaitFor::BlockInclusion);

	let res = watcher.run().await?;
	let res = res.unwrap();
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

	println!("Transaction Execute finished correctly");

	Ok(())
}
