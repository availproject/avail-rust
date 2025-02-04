use avail_rust::{block::to_ascii, prelude::*};
use std::time::SystemTime;

type DataSubmissionCall = avail::data_availability::calls::types::SubmitData;
type ApplicationKeyCreatedEvent = avail::data_availability::events::ApplicationKeyCreated;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = account::alice();

	// Application Key Creation
	let time = std::format!("{:?}", SystemTime::now());
	let key = time.into_bytes();
	let tx = sdk.tx.data_availability.create_application_key(key);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions must be successful");

	let events = res.events.as_ref().unwrap();
	let event = events.find_first::<ApplicationKeyCreatedEvent>().unwrap();
	let Some(event) = event else {
		return Err("Failed to get Application Key Created Event".into());
	};
	let app_id = event.id.0;

	// Data Submission
	let data = String::from("My Data").into_bytes();
	let options = Options::new().app_id(app_id);
	let tx = sdk.tx.data_availability.submit_data(data);
	let res = tx.execute_and_watch_inclusion(&account, Some(options)).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions must be successful");

	println!(
		"Block Hash: {:?}, Block Number: {}, Tx Hash: {:?}, Tx Index: {}",
		res.block_hash, res.block_number, res.tx_hash, res.tx_index
	);

	// Events
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

	// Decoding
	let decoded = res.decode_as::<DataSubmissionCall>().await?;
	let Some(decoded) = decoded else {
		return Err("Failed to get Data Submission Call data".into());
	};

	let data = to_ascii(decoded.data.0).unwrap();
	println!("Call data: {:?}", data);

	Ok(())
}
