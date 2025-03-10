use avail_rust::{da_commitments::DaCommitmentBuilder, error::ClientError, prelude::*};

type DataSubmissionWithCommitmentsCall = avail::data_availability::calls::types::SubmitDataWithCommitments;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;
	let account = account::bob();

	// Data to be submitted
	let data = b"Testing submit_data_wth_commitments of Avail DA".to_vec();
	let commitments = DaCommitmentBuilder::new(data.clone()).build().unwrap();

	let options = Options::new().app_id(2);
	let tx = sdk.tx.data_availability.submit_data_with_commitments(data, commitments);
	let res = tx.execute_and_watch_inclusion(&account, options).await?;
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
	let decoded = res.decode_as::<DataSubmissionWithCommitmentsCall>().await?;
	let Some(decoded) = decoded else {
		return Err("Failed to get Data Submission Call data".into());
	};

	let data = to_ascii(decoded.data.0).unwrap();
	println!("Call data: {:?}", data);

	println!("Data Submission with commitments completed correctly");

	Ok(())
}
