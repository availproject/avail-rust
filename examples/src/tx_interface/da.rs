use avail_rust::error::ClientError;

pub async fn run() -> Result<(), ClientError> {
	println!("da_submit_data");
	submit_data::run().await?;
	println!("da_create_application_key");
	create_application_key::run().await?;

	Ok(())
}

mod submit_data {
	use avail_rust::{
		prelude::*,
		transactions::{DataAvailabilityCalls, DataAvailabilityEvents},
	};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Alice")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let data = String::from("My Awesome Data").as_bytes().to_vec();

		let options = Options::new().nonce(Nonce::BestBlock).app_id(1);
		let tx = sdk.tx.data_availability.submit_data(data);
		let res = tx.execute_and_watch_inclusion(&account, Some(options)).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.as_ref().unwrap();
		assert_eq!(events.has::<DataAvailabilityEvents::DataSubmitted>(), Some(true), "");
		assert_eq!(res.is::<DataAvailabilityCalls::SubmitData>().await.unwrap(), true, "");

		Ok(())
	}
}

mod create_application_key {
	use avail_rust::{prelude::*, transactions::DataAvailabilityEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Alice")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let key = String::from("My Tx Interface Key").as_bytes().to_vec();

		let tx = sdk.tx.data_availability.create_application_key(key);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(
			events.has::<DataAvailabilityEvents::ApplicationKeyCreated>(),
			Some(true),
			""
		);
		Ok(())
	}
}
