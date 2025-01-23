use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;
	// Setup
	let data = String::from("My Data").into_bytes();
	let tx = sdk.tx.data_availability.submit_data(data);
	let res = tx.execute_and_watch_inclusion(&SDK::alice()?, None).await?;
	match res.is_successful(&sdk.online_client) {
		Some(x) => x?,
		None => panic!("Failed to decode events."),
	};

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
	let total_event_counts: usize = events.iter().map(|e| e.iter().count()).sum();
	println!(
		"Events Groups count: {}. Total events count: {}",
		events.len(),
		total_event_counts
	);

	// Fetching all events from a block for a specific transaction
	let events = block.events(Some(res.tx_index)).await?;
	let total_event_counts: usize = events.iter().map(|e| e.iter().count()).sum();
	println!(
		"Events Groups count: {}. Total events count: {}",
		events.len(),
		total_event_counts
	);

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
	Example Output:

	Tx Pallet name: DataAvailability, Tx Name: submit_data
	Tx Pallet name: DataAvailability, Tx Name: submit_data
	Tx Call Data: BoundedVec([77, 121, 32, 68, 97, 116, 97])
	Tx Hash: 0xf0439041a4138217d042c4d2ef75657b3b5c98cfaa2e85dcca94a47a65472a31, Tx Index: 1, Data [77, 121, 32, 68, 97, 116, 97], Tx Signer: [1, 196, 39, 196, 81, 65, 82, 28, 80, 157, 36, 247, 217, 186, 203, 75, 149, 165, 250, 33, 198, 34, 57, 111, 250, 41, 65, 249, 148, 110, 42, 154, 19, 117, 38, 169, 162, 154, 87, 118, 88, 122, 225, 157, 246, 91, 82, 9, 171, 86, 42, 197, 63, 218, 111, 241, 64, 24, 13, 155, 47, 143, 160, 74, 132], App Id: 0
	Events Groups count: 3. Total events count: 9
	Events Groups count: 1. Total events count: 7
	Next App Id: 10
*/
