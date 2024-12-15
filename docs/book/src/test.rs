use avail_rust::{error::ClientError, Block, SDK};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new_http("http://localhost:9944").await?;
	let alice = SDK::alice()?;

	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);
	let res = tx.http_execute_and_watch_inclusion(&alice, None).await?;

	println!("Genesis Hash {:?}", sdk.online_client.genesis_hash());
	println!("Tx Hash {:?}", res.tx_hash);

	let block = Block::new(&sdk.online_client, res.block_hash).await?;
	let a = block.data_submissions_all();
	dbg!(a);
	block.transaction_by_app_id(0);
	println!("Ok");
	Ok(())
}
