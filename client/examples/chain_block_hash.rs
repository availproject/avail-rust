use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Block Hash
	let best_block_hash = client.best().block_hash().await?;
	let finalized_block_hash = client.finalized().block_hash().await?;
	let block_hash = client
		.chain()
		.block_hash(Some(2000000))
		.await?
		.expect("Should be there");
	println!("Best Block Hash:              {:?}", best_block_hash);
	println!("Finalized Block Hash:         {:?}", finalized_block_hash);
	println!("Block Hash for block 2000000: {:?}", block_hash);

	Ok(())
}

/*
	Expected Output:

	Best Block Hash:              0x28e446be3885c6666ec9ca6531844870e51facc8aff5ed6e9337a10bff4110aa
	Finalized Block Hash:         0xac52286fd7f0f308e58c16746f6076f8602a44cda9d6b9f54378bb9109818465
	Block Hash for block 2000000: 0x6831d536cc3d6408a41a1e50d66f4f48c9c2ed5ffc7cfa7505a5f0251365428f
*/
