use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Block Height
	let best_block_height = client.best().block_height().await?;
	let finalized_block_height = client.finalized().block_height().await?;
	let block_height = client
		.chain()
		.block_height("0x6831d536cc3d6408a41a1e50d66f4f48c9c2ed5ffc7cfa7505a5f0251365428f")
		.await?
		.expect("Should be there");
	println!(
		"Best Block Height: {}, Finalized Block Height: {:?}, Block Height for block 0x6831d536cc3d6408a41a1e50d66f4f48c9c2ed5ffc7cfa7505a5f0251365428f: {:?}",
		best_block_height, finalized_block_height, block_height
	);

	Ok(())
}

/*
	Expected Output:

	Best Block Height: 2503770, Finalized Block Height: 2503769, Block Height for block 0x6831d536cc3d6408a41a1e50d66f4f48c9c2ed5ffc7cfa7505a5f0251365428f: 2000000
*/
