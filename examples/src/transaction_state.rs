use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	// Transaction will be signed, and sent.
	//
	// There is no guarantee that the transaction was executed at all. It might have been
	// dropped or discarded for various reasons. The caller is responsible for querying future
	// blocks in order to determine the execution status of that transaction.
	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);
	let tx_hash = tx.execute(&account::alice(), Options::new().app_id(1)).await?;
	println!("Tx Hash: {:?}", tx_hash);

	let result = loop {
		let res = sdk.client.transaction_state(&tx_hash, false).await?;
		if !res.is_empty() {
			break res;
		}
		std::thread::sleep(std::time::Duration::from_secs(1));
	};
	assert_eq!(result.len(), 1);

	println!(
		"Block Hash: {:?}, Block Height: {}, Tx Hash: {:?}, Tx Index {}",
		result[0].block_hash, result[0].block_height, result[0].tx_hash, result[0].tx_index
	);
	println!(
		"Pallet Index: {:?}, Call Index: {}, Tx Success: {:?}, Is Finalized {}",
		result[0].pallet_index, result[0].call_index, result[0].tx_success, result[0].is_finalized
	);

	println!("Transaction State finished correctly");

	Ok(())
}
