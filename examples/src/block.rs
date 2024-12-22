use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;
	// Setup
	let data = String::from("My Data").into_bytes();
	let tx = sdk.tx.data_availability.submit_data(data);
	let res = tx.execute_and_watch_inclusion(&SDK::alice()?, None).await?;
	res.is_successful(&sdk.online_client)?;

	// Fetching
	// Fetching best block
	_ = Block::new_best_block(&sdk.online_client, &sdk.rpc_client).await?;

	// Fetching finalized block
	_ = Block::new_finalized_block(&sdk.online_client, &sdk.rpc_client).await?;

	// Fetching block with hex string or hash
	let hex_string = std::format!("{:?}", res.block_hash);
	let block_hash = avail_rust::utils::hex_string_to_h256(&hex_string)?;
	_ = Block::new(&sdk.online_client, block_hash).await?;

	// Fetching block with block number
	let block_number = 0;
	_ = Block::from_block_number(&sdk.online_client, &sdk.rpc_client, block_number);

	// Transactions
	let block = Block::new(&sdk.online_client, res.block_hash).await?;

	// Filtering by Transaction Index
	let tx = block
		.transaction_by_index(res.tx_index)
		.ok_or(String::from("Failed to find tx"))?;
	println!(
		"Tx Pallet name: {}, Tx Name: {}",
		tx.pallet_name()?,
		tx.variant_name()?,
	);

	// Filtering by Transaction Index with Call Data
	use avail::data_availability::calls::types::SubmitData;
	let tx = block
		.transaction_by_index_static::<SubmitData>(res.tx_index)
		.ok_or(String::from("Failed to find tx"))?;
	println!(
		"Tx Pallet name: {}, Tx Name: {}",
		tx.details.pallet_name()?,
		tx.details.variant_name()?,
	);
	println!("Tx Call Data: {:?}", tx.value.data);
	/*
	Available methods:
		transaction_all_static
		transaction_count
		transaction_by_signer
		transaction_by_signer_static
		transaction_by_index
		transaction_by_index_static
		transaction_by_hash
		transaction_by_hash_static
		transaction_by_app_id
		transaction_by_app_id_static
	*/

	// Data Submission
	// Filtering by Transaction Index
	let ds = block
		.data_submissions_by_index(res.tx_index)
		.ok_or(String::from("Failed to find ds"))?;
	println!(
		"Tx Hash: {:?}, Tx Index: {}, Data {:?}, Tx Signer: {:?}, App Id: {}",
		ds.tx_hash, ds.tx_index, ds.data, ds.tx_signer, ds.app_id
	);
	/*
	Available methods:
		data_submissions_all
		data_submissions_by_signer
		data_submissions_by_index
		data_submissions_by_hash
		data_submissions_by_app_id
	*/

	// Fetching all events from a block
	let events = block.events(None).await?;
	println!("Events count: {}", events.len());

	// Fetching all events from a block for a specific transaction
	let events = block.events(Some(res.tx_index)).await?;
	println!("Events count: {}", events.len());

	// Finding the tx index with tx hash
	let tx_index = block
		.transaction_hash_to_index(res.tx_hash)
		.ok_or(String::from("Failed to find index"))?;
	assert_eq!(tx_index, res.tx_index);

	let address = avail::storage().data_availability().next_app_id();
	let app_id = block.storage_fetch_or_default(&address).await?.0;
	println!("Next App Id: {}", app_id);
	/*
	Available methods:
		storage_fetch
		storage_fetch_or_default
		storage_iter
	*/

	Ok(())
}

/*
	Expected Output:

	Tx Pallet name: DataAvailability, Tx Name: submit_data
	Tx Pallet name: DataAvailability, Tx Name: submit_data
	Tx Call Data: BoundedVec([77, 121, 32, 68, 97, 116, 97])
	Tx Hash: 0x0397b6f23ba5e534771b6963a51886dc475724b8fa2b0393e89eb48ddf2a6d91, Tx Index: 1, Data [77, 121, 32, 68, 97, 116, 97], Tx Signer: [1, 148, 140, 212, 44, 213, 20, 183, 36, 162, 143, 92, 16, 11, 80, 252, 38, 49, 129, 229, 159, 46, 165, 127, 124, 128, 24, 236, 40, 108, 197, 13, 67, 105, 12, 248, 226, 20, 13, 70, 68, 134, 1, 171, 49, 14, 2, 122, 87, 200, 132, 11, 87, 244, 85, 175, 237, 125, 233, 88, 211, 168, 231, 118, 135], App Id: 0
	Events count: 3
	Events count: 1
	Next App Id: 10
*/
