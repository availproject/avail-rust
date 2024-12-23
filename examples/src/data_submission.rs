use avail_rust::prelude::*;

type DataSubmissionCall = avail::data_availability::calls::types::SubmitData;
type ApplicationKeyCreatedEvent = avail::data_availability::events::ApplicationKeyCreated;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;
	let online_client = &sdk.online_client;

	let account = SDK::alice()?;

	// Application Key Creation
	let key = String::from("My Key").into_bytes();
	let tx = sdk.tx.data_availability.create_application_key(key);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	res.is_successful(&online_client)?;

	let Some(event) = res.find_first_event::<ApplicationKeyCreatedEvent>() else {
		return Err("Failed to get Application Key Created Event".into());
	};
	let app_id = event.id.0;

	// Data Submission
	let data = String::from("My Data").into_bytes();
	let options = Options::new().app_id(app_id);
	let tx = sdk.tx.data_availability.submit_data(data);
	let res = tx
		.execute_and_watch_inclusion(&account, Some(options))
		.await?;
	res.is_successful(&online_client)?;

	println!(
		"Block Hash: {:?}, Block Number: {}, Tx Hash: {:?}, Tx Index: {}",
		res.block_hash, res.block_number, res.tx_hash, res.tx_index
	);

	let Some(call_data) = res.get_call_data::<DataSubmissionCall>(online_client).await else {
		return Err("Failed to get Data Submission Call data".into());
	};
	println!("Call data: {:?}", call_data.data);

	// Getting Data Submission from Block #1
	let block = Block::new(online_client, res.block_hash).await?;

	// data_submissions_by_signer, data_submissions_by_index, data_submissions_by_hash, data_submissions_by_app_id
	let data_submissions = block.data_submissions_all();
	for ds in data_submissions {
		println!(
			"Tx Hash: {:?}, Tx Index: {}, Data {:?}, Tx Signer: {:?}, App Id: {}",
			ds.tx_hash, ds.tx_index, ds.data, ds.tx_signer, ds.app_id
		);

		println!("Ascii data: {}", ds.to_ascii().expect("qed"));
	}

	// Getting Data Submission from Block #2
	for tx in block.transaction_all_static::<DataSubmissionCall>() {
		println!("Call data: {:?}", tx.value.data);

		let ds = DataSubmission::from_static(tx);
		println!(
			"Tx Hash: {:?}, Tx Index: {}, Data {:?}, Tx Signer: {:?}, App Id: {}",
			ds.tx_hash, ds.tx_index, ds.data, ds.tx_signer, ds.app_id
		);

		println!("Ascii data: {}", ds.to_ascii().expect("qed"));
	}

	Ok(())
}

/*
	Example Output:

	Block Hash: 0x95db9a398c60358ade0504bef2eb6bf77c6cf05dee0525f43516cefab763b60f, Block Number: 485, Tx Hash: 0x6b4abd33d1452c0aa3d2fb9f4f4bbeb4f9d2d20b0b5bfb55696eea551974dcd3, Tx Index: 1
	Call data: BoundedVec([77, 121, 32, 68, 97, 116, 97])
	Tx Hash: 0x6b4abd33d1452c0aa3d2fb9f4f4bbeb4f9d2d20b0b5bfb55696eea551974dcd3, Tx Index: 1, Data [77, 121, 32, 68, 97, 116, 97], Tx Signer: [1, 202, 251, 108, 14, 95, 87, 191, 103, 174, 23, 201, 117, 10, 32, 139, 45, 55, 84, 14, 101, 67, 180, 132, 224, 20, 88, 26, 241, 244, 83, 32, 2, 45, 179, 41, 23, 165, 8, 7, 65, 52, 143, 32, 5, 60, 109, 132, 22, 89, 98, 198, 151, 88, 202, 92, 229, 70, 49, 127, 101, 254, 166, 81, 131], App Id: 12
	Ascii data: My Data
	Call data: BoundedVec([77, 121, 32, 68, 97, 116, 97])
	Tx Hash: 0x6b4abd33d1452c0aa3d2fb9f4f4bbeb4f9d2d20b0b5bfb55696eea551974dcd3, Tx Index: 1, Data [77, 121, 32, 68, 97, 116, 97], Tx Signer: [1, 202, 251, 108, 14, 95, 87, 191, 103, 174, 23, 201, 117, 10, 32, 139, 45, 55, 84, 14, 101, 67, 180, 132, 224, 20, 88, 26, 241, 244, 83, 32, 2, 45, 179, 41, 23, 165, 8, 7, 65, 52, 143, 32, 5, 60, 109, 132, 22, 89, 98, 198, 151, 88, 202, 92, 229, 70, 49, 127, 101, 254, 166, 81, 131], App Id: 12
	Ascii data: My Data
*/
