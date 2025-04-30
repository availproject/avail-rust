/* use avail_rust::prelude::*;
use std::time::SystemTime;

type ApplicationKeyCreatedEvent = avail::data_availability::events::ApplicationKeyCreated;

pub async fn run() -> Result<(), ClientError> {
	run_transaction().await?;
	run_block().await?;

	println!("HTTP Client finished correctly");
	Ok(())
}

pub async fn run_transaction() -> Result<(), ClientError> {
	let sdk = SDK::new_http(SDK::local_http_endpoint()).await?;

	let account = account::alice();

	// Application Key Creation
	let time = std::format!("{:?}", SystemTime::now());
	let key = time.into_bytes();
	let tx = sdk.tx.data_availability.create_application_key(key);
	let res = tx.execute_and_watch(&account, Options::default()).await?;
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
	let res = tx.execute_and_watch(&account, options).await?;
	assert_eq!(res.is_successful(), Some(true));

	Ok(())
}

pub async fn run_block() -> Result<(), ClientError> {
	let sdk = SDK::new_http(SDK::turing_http_endpoint()).await?;
	let block_hash = H256::from_hex("0x94746ba186876d7407ee618d10cb6619befc59eeb173cacb00c14d1ff492fc58")?;

	let block = Block::new(&sdk.client, block_hash).await?;

	// All Transactions
	let block_transactions = block.transactions(Filter::default());
	assert_eq!(block_transactions.len(), 9, "Transaction count must be 9");

	// Printout Block Transactions
	for tx in block_transactions.iter().take(2) {
		println!("Tx Index: {}", tx.tx_index());
	}

	let blobs = block.data_submissions(Filter::default());
	assert_eq!(blobs.len(), 4, "Blobs must present 4 times");

	// Printout All Block Blobs
	for blob in blobs.iter().take(2) {
		println!("Tx Index: {}", blob.tx_index,);
	}

	Ok(())
}
 */
