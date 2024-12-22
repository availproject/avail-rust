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

		let options = Options::new().nonce(Nonce::BestBlockAndTxPool).app_id(1);
		let tx = sdk.tx.data_availability.submit_data(data);
		let res = tx
			.execute_and_watch_inclusion(&account, Some(options))
			.await?;
		res.is_successful(&sdk.online_client)?;

		res.print_debug();
		let Some(event) = res.find_first_event::<DataAvailabilityEvents::DataSubmitted>() else {
			return Err("Failed to find DataSubmitted event".into());
		};
		dbg!(event);
		let Some(data) = res
			.get_call_data::<DataAvailabilityCalls::SubmitData>(&sdk.online_client)
			.await
		else {
			return Err("Failed to find SubmitDataCall data".into());
		};
		dbg!(data);

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
		let key = String::from("MyAwesomeKey").as_bytes().to_vec();

		let tx = sdk.tx.data_availability.create_application_key(key);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		res.is_successful(&sdk.online_client)?;

		res.print_debug();
		let Some(event) = res.find_first_event::<DataAvailabilityEvents::ApplicationKeyCreated>()
		else {
			return Err("Failed to find ApplicationKeyCreated event".into());
		};
		dbg!(event);

		Ok(())
	}
}
