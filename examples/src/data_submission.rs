use avail_rust::prelude::*;

type DataSubmissionCall = avail::data_availability::calls::types::SubmitData;
type ApplicationKeyCreatedEvent = avail::data_availability::events::ApplicationKeyCreated;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = account::alice();

	// Application Key Creation
	let key = String::from("My Key").into_bytes();
	let tx = sdk.tx.data_availability.create_application_key(key);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions must be successful");

	let events = res.events.as_ref().unwrap();
	let event = events.find_first::<ApplicationKeyCreatedEvent>().unwrap();
	let Ok(event) = event else {
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

	let decoded = res.decode_as::<DataSubmissionCall>().await?;
	let Some(decoded) = decoded else {
		return Err("Failed to get Data Submission Call data".into());
	};
	println!("Call data: {:?}", decoded.data);

	Ok(())
}
