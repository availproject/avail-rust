use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Chain Info
	let chain_info = client.chain().chain_info().await?;
	println!("Best      Hash: {:?}, Height: {}", chain_info.best_hash, chain_info.best_height);
	println!("Finalized Hash: {:?}, Height: {}", chain_info.finalized_hash, chain_info.finalized_height);
	println!("Genesis   Hash: {:?}", chain_info.genesis_hash);

	Ok(())
}

/*
	Expected Output:

	Best      Hash: 0x1805b4e57f32552854eb279522cdb79e1014bb374c4024d43fe2fa03f039ea16, Height: 2503885
	Finalized Hash: 0x15d09b2e7e3286d284e0c4392770c7269cab48b75aa60c3c67306a0ce38027b4, Height: 2503884
	Genesis   Hash: 0xd3d2f3a3495dc597434a99d7d449ebad6616db45e4e4f178f31cc6fa14378b70
*/
