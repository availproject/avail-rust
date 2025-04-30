/* use avail::data_availability::{calls::types::SubmitData, events::DataSubmitted};
use avail_rust::prelude::*;
use std::time::Duration;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	// Transaction will be signed, and sent.
	//
	// There is no guarantee that the transaction was executed at all. It might have been
	// dropped or discarded for various reasons. The caller is responsible for querying future
	// blocks in order to determine the execution status of that transaction.
	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);
	let info = tx.execute(&account::alice(), Options::new().app_id(1)).await?;
	println!("Tx Hash: {:?}", info.tx_hash);

	// Checking if the transaction was included
	let sleep_duration = Duration::from_secs(10);
	// or `info.find_block_id_best_block(sleep_duration).await?.unwrap();`
	let block_id = info.find_block_id(sleep_duration).await?.unwrap();
	let res = sdk.client.block_transaction_at(info.tx_hash, block_id.hash).await?;
	let res = res.unwrap();

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
 */
