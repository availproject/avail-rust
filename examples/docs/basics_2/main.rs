use avail_rust::error::ClientError;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	// RPC Connection
	// ANCHOR: connection
	use avail_rust::sdk::reconnecting_api;

	let endpoint = "ws://127.0.0.1:9944";
	let (online_client, rpc_client) = reconnecting_api(endpoint).await?;
	// ANCHOR_END: connection

	// Accounts
	// ANCHOR: accounts
	use avail_rust::SDK;

	let account = SDK::alice()?;
	// ANCHOR_END: accounts

	// Payload
	// ANCHOR: payload
	use avail_rust::avail::runtime_types::bounded_collections::bounded_vec::BoundedVec;

	let data = String::from("My Data").into_bytes();
	let data = BoundedVec(data);
	let payload = avail_rust::avail::tx()
		.data_availability()
		.submit_data(data);
	// ANCHOR_END: payload

	// Transaction Params, Signature, Submission
	// ANCHOR: signsend
	use avail_rust::transaction::utils::sign_and_send;

	let tx_hash = sign_and_send(&online_client, &rpc_client, &account, &payload, None).await?;
	// ANCHOR_END: signsend

	// Watcher
	// ANCHOR: watcher
	use avail_rust::{transaction::utils::watch, WaitFor};

	let tx_details = watch(&online_client, tx_hash, WaitFor::BlockInclusion, Some(3)).await?;
	println!("Transaction was found.");
	println!("Block Hash: {:?}", tx_details.block_hash); // Block Hash: 0x61415b6012005665bac0cf8575a94e509d079a762be2ba6a71a04633efd01c1b
	println!("Block Number: {:?}", tx_details.block_number); // Block Number: 200
	println!("Tx Hash: {:?}", tx_details.tx_hash); // Tx Hash: 0x01651a93d55bde0f258504498d4f2164416df5331794d9c905d4c8711d9537ef
	println!("Tx Index: {:?}", tx_details.tx_index); // Tx Index: 1

	println!("Event count: {}", tx_details.events.iter().count()); // Event count: 7
	tx_details.is_successful(&online_client)?;
	// ANCHOR_END: watcher

	Ok(())
}
