use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Block Info
	let best_block_info = client.best().block_info().await?;
	let finalized_block_info = client.finalized().block_info().await?;
	println!("Best      Hash: {:?}, Height: {}", best_block_info.hash, best_block_info.height);
	println!("Finalized Hash: {:?}, Height: {}", finalized_block_info.hash, finalized_block_info.height);

	Ok(())
}

/*
	Expected Output:

	Best      Hash: 0x6378669297b6eb2127b32462c6f1b265a36eb3810a9f4c532ff603864e9ecdc5, Height: 2503892
	Finalized Hash: 0x60863650fa00c9203c28b832c1c55de46cab95992298ac59618e592117d49a2f, Height: 2503891
*/
