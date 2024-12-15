use avail_rust::error::ClientError;
use avail_rust::utils::http_watch_transaction;
use avail_rust::Block;
use avail_rust::SDK;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new_http("http://localhost:9944").await?;
	let alice = SDK::alice()?;

	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);
	let tx_hash = tx.http_execute_and_forget(&alice, None).await?;

	println!("Genesis Hash {:?}", sdk.online_client.genesis_hash());
	println!("Tx Hash {:?}", tx_hash);

	let res = http_watch_transaction(
		&sdk.online_client,
		&sdk.rpc_client,
		tx_hash,
		avail_rust::WaitFor::BlockInclusion,
		Some(3),
	)
	.await
	.unwrap();

	let block = Block::new(&sdk.online_client, res.block_hash).await?;
	let a = block.data_submissions_all();
	dbg!(a);
	block.transaction_by_app_id(0);
	println!("Ok");
	Ok(())
}
