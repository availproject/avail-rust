use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Block Header
	let best_block_header = client.best().block_header().await?;
	let finalized_block_header = client.finalized().block_header().await?;
	let block_header = client
		.chain()
		.block_header(Some(2000000))
		.await?
		.expect("Should be there");
	println!("Best Block Header Data Root:              {:?}", best_block_header.data_root(),);
	println!("Finalized Block Header Data Root:         {:?}", finalized_block_header.data_root(),);
	println!("Block Header for block 2000000 Data Root: {:?}", block_header.data_root());

	Ok(())
}

/*
	Expected Output:

	Best Block Header Data Root:              0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5
	Finalized Block Header Data Root:         0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5
	Block Header for block 2000000 Data Root: 0xad3228b676f7d3cd4284a5443f17f1962b36e491b30a40b2405849e597ba5fb5
*/
