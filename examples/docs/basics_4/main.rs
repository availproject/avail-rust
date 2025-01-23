use avail_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	// RPC Connection
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	// Accounts
	let account = SDK::alice()?;

	// ANCHOR: success
	let key = String::from("My Data").into_bytes();
	let tx = sdk.tx.data_availability.create_application_key(key);
	let tx_details = tx.execute_and_watch_inclusion(&account, None).await?;
	// Checking if the transaction was successful
	match tx_details.is_successful(&sdk.online_client) {
		Some(x) => x?,
		None => panic!("Failed to decode events."),
	};
	// ANCHOR_END: success

	// Finding ApplicationKeyCreated event
	// ANCHOR: event
	use avail_rust::avail::data_availability::events::ApplicationKeyCreated;
	let Some(event) = tx_details.find_first_event::<ApplicationKeyCreated>() else {
		return Err("Failed to find event".into());
	};
	println!("App id: {}", event.id.0);
	// ANCHOR_END: event

	// Fetching block
	// ANCHOR: block
	let block = Block::new(&sdk.online_client, tx_details.block_hash).await?;
	let tx_count = block.transactions.iter().count();
	println!("Transaction count in a block: {}", tx_count);
	// ANCHOR_END: block

	// Using custom payload with Transaction object
	// ANCHOR: custompayload
	// ! Check Transaction 1(basics_1) or Transaction 2(basics_2) example for custom payload. !
	let data = String::from("Data").into_bytes();
	let data = BoundedVec(data);
	let payload = avail_rust::avail::tx()
		.data_availability()
		.submit_data(data);
	let tx = Transaction::new(sdk.online_client.clone(), sdk.rpc_client.clone(), payload);
	let tx_details = tx.execute_and_watch_inclusion(&account, None).await?;
	// Checking if the transaction was successful
	match tx_details.is_successful(&sdk.online_client) {
		Some(x) => x?,
		None => panic!("Failed to decode events."),
	};
	// ANCHOR_END: custompayload

	Ok(())
}
