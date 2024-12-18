use avail_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	// RPC Connection
	// ANCHOR: connection
	let sdk = SDK::new(SDK::local_endpoint()).await?;
	// ANCHOR_END: connection

	// Accounts
	let account = SDK::alice()?;

	// Payload
	// ANCHOR: payload
	let data = String::from("My Data").into_bytes();
	let tx = sdk.tx.data_availability.submit_data(data);
	// ANCHOR_END: payload

	// Transaction Params, Signature, Submission, Watcher
	// ANCHOR: signsend
	let tx_details = tx.execute_and_watch_inclusion(&account, None).await?;
	println!("Transaction was found.");
	println!("Block Hash: {:?}", tx_details.block_hash); // Block Hash: 0x61415b6012005665bac0cf8575a94e509d079a762be2ba6a71a04633efd01c1b
	println!("Block Number: {:?}", tx_details.block_number); // Block Number: 200
	println!("Tx Hash: {:?}", tx_details.tx_hash); // Tx Hash: 0x01651a93d55bde0f258504498d4f2164416df5331794d9c905d4c8711d9537ef
	println!("Tx Index: {:?}", tx_details.tx_index); // Tx Index: 1

	println!("Event count: {}", tx_details.events.iter().count()); // Event count: 7
	tx_details.is_successful(&sdk.online_client)?;
	// ANCHOR_END: signsend

	Ok(())
}
