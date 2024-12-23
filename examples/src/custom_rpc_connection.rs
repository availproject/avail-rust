use avail_rust::prelude::*;
use std::time::Duration;
use subxt::backend::rpc::{
	reconnecting_rpc_client::{ExponentialBackoff, RpcClient as ReconnectingRpcClient},
	RpcClient,
};

type DataSubmissionCall = avail::data_availability::calls::types::SubmitData;
type ApplicationKeyCreatedEvent = avail::data_availability::events::ApplicationKeyCreated;

pub async fn run() -> Result<(), ClientError> {
	let rpc_client = ReconnectingRpcClient::builder()
		.retry_policy(
			ExponentialBackoff::from_millis(1000)
				.max_delay(Duration::from_secs(3))
				.take(3),
		)
		.build(SDK::local_endpoint())
		.await
		.map_err(|e| e.to_string())?;
	let rpc_client = RpcClient::new(rpc_client);
	let online_client = AOnlineClient::from_rpc_client(rpc_client.clone()).await?;

	let sdk = SDK::new_custom(online_client, rpc_client).await?;
	let online_client = &sdk.online_client;

	let account = SDK::alice()?;

	// Application Key Creation
	let key = String::from("My Key Custom").into_bytes();
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

	Block Hash: 0xb8b08997ab5e45c834e28c0ccab4d73eb94b95814500049b11fcf72f5e999c70, Block Number: 475, Tx Hash: 0xd7a62ff853acac85c042f2f42b123b9fab73f0290105107dd081d2aa3785877d, Tx Index: 1
	Call data: BoundedVec([77, 121, 32, 68, 97, 116, 97])
	Tx Hash: 0xd7a62ff853acac85c042f2f42b123b9fab73f0290105107dd081d2aa3785877d, Tx Index: 1, Data [77, 121, 32, 68, 97, 116, 97], Tx Signer: [1, 48, 143, 125, 91, 21, 60, 171, 139, 249, 159, 63, 160, 208, 148, 213, 254, 200, 128, 124, 163, 191, 18, 226, 173, 62, 86, 20, 219, 248, 100, 60, 71, 220, 201, 196, 171, 104, 203, 65, 100, 96, 66, 56, 89, 109, 100, 63, 40, 151, 206, 46, 200, 73, 10, 63, 154, 226, 232, 161, 146, 143, 249, 94, 142], App Id: 11
	Ascii data: My Data
	Call data: BoundedVec([77, 121, 32, 68, 97, 116, 97])
	Tx Hash: 0xd7a62ff853acac85c042f2f42b123b9fab73f0290105107dd081d2aa3785877d, Tx Index: 1, Data [77, 121, 32, 68, 97, 116, 97], Tx Signer: [1, 48, 143, 125, 91, 21, 60, 171, 139, 249, 159, 63, 160, 208, 148, 213, 254, 200, 128, 124, 163, 191, 18, 226, 173, 62, 86, 20, 219, 248, 100, 60, 71, 220, 201, 196, 171, 104, 203, 65, 100, 96, 66, 56, 89, 109, 100, 63, 40, 151, 206, 46, 200, 73, 10, 63, 154, 226, 232, 161, 146, 143, 249, 94, 142], App Id: 11
	Ascii data: My Data
*/
