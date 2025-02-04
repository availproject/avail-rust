use avail_rust::{account, avail, error::ClientError, transaction::HTTP, Options, SDK};

type ApplicationKeyCreatedEvent = avail::data_availability::events::ApplicationKeyCreated;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new_http(SDK::local_http_endpoint()).await?;

	let account = account::alice();

	// Application Key Creation
	let key = String::from("My Key Http").into_bytes();
	let tx = sdk.tx.data_availability.create_application_key(key);
	let res = tx.execute_and_watch_inclusion(&account, Options::default()).await?;
	assert_eq!(res.is_successful(), Some(true));

	let events = res.events.unwrap();
	let event = events.find_first::<ApplicationKeyCreatedEvent>().unwrap();
	let Some(event) = event else {
		return Err("Failed to get Application Key Created Event".into());
	};
	let app_id = event.id.0;

	// Data Submission
	let data = String::from("My Data").into_bytes();
	let options = Options::new().app_id(app_id);
	let tx = sdk.tx.data_availability.submit_data(data);
	let res = tx.execute_and_watch_inclusion(&account, options).await?;
	assert_eq!(res.is_successful(), Some(true));

	Ok(())
}
