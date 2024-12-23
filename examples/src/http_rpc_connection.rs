use avail_rust::{
	avail,
	block::{Block, DataSubmission},
	error::ClientError,
	transaction::HTTP,
	Options, SDK,
};

type DataSubmissionCall = avail::data_availability::calls::types::SubmitData;
type ApplicationKeyCreatedEvent = avail::data_availability::events::ApplicationKeyCreated;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new_http(SDK::local_http_endpoint()).await?;
	let online_client = &sdk.online_client;

	let account = SDK::alice()?;

	// Application Key Creation
	let key = String::from("My Key Http").into_bytes();
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

	Block Hash: 0x434f28e191b0b2bf4e9e379fd21d8a53d52933f7f2df5829f36ec221c583b005, Block Number: 502, Tx Hash: 0xc5dfa3c4b62280febc3cdb9638650441596dbb02c427ebb77d25201b6e52e2ec, Tx Index: 1
	Call data: BoundedVec([77, 121, 32, 68, 97, 116, 97])
	Tx Hash: 0xc5dfa3c4b62280febc3cdb9638650441596dbb02c427ebb77d25201b6e52e2ec, Tx Index: 1, Data [77, 121, 32, 68, 97, 116, 97], Tx Signer: [1, 152, 76, 87, 244, 106, 83, 18, 214, 247, 138, 6, 162, 56, 34, 56, 182, 50, 22, 174, 89, 219, 133, 176, 244, 24, 155, 213, 201, 63, 146, 181, 36, 247, 60, 134, 221, 14, 102, 58, 148, 247, 218, 33, 47, 13, 103, 227, 186, 13, 221, 104, 243, 209, 74, 163, 74, 212, 168, 101, 255, 150, 88, 251, 142], App Id: 13
	Ascii data: My Data
	Call data: BoundedVec([77, 121, 32, 68, 97, 116, 97])
	Tx Hash: 0xc5dfa3c4b62280febc3cdb9638650441596dbb02c427ebb77d25201b6e52e2ec, Tx Index: 1, Data [77, 121, 32, 68, 97, 116, 97], Tx Signer: [1, 152, 76, 87, 244, 106, 83, 18, 214, 247, 138, 6, 162, 56, 34, 56, 182, 50, 22, 174, 89, 219, 133, 176, 244, 24, 155, 213, 201, 63, 146, 181, 36, 247, 60, 134, 221, 14, 102, 58, 148, 247, 218, 33, 47, 13, 103, 227, 186, 13, 221, 104, 243, 209, 74, 163, 74, 212, 168, 101, 255, 150, 88, 251, 142], App Id: 13
	Ascii data: My Data
*/
