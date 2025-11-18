use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// By default it subscribes to finalized block
	let mut sub = Sub::new(client.clone());
	let next_info = sub.next().await?;
	let prev_info = sub.prev().await?;
	println!("Finalized Next:      Block Height: {}, Block Hash: {:?}", next_info.height, next_info.hash);
	println!("Finalized Previous:  Block Height: {}, Block Hash: {:?}", prev_info.height, prev_info.hash);

	// Best Blocks
	let mut sub = Sub::new(client.clone());
	sub.use_best_block(true);
	let next_info = sub.next().await?;
	let prev_info = sub.prev().await?;
	println!("Best Next:           Block Height: {}, Block Hash: {:?}", next_info.height, next_info.hash);
	println!("Best Previous:       Block Height: {}, Block Hash: {:?}", prev_info.height, prev_info.hash);

	// Historical Blocks
	let mut sub = Sub::new(client.clone());
	sub.set_block_height(2000000);
	let next_info = sub.next().await?;
	let prev_info = sub.prev().await?;
	println!("Historical Next:     Block Height: {}, Block Hash: {:?}", next_info.height, next_info.hash);
	println!("Historical Previous: Block Height: {}, Block Hash: {:?}", prev_info.height, prev_info.hash);

	Ok(())
}

/*
	Expected Output:

	Finalized Next:      Block Height: 2504105, Block Hash: 0x9bccc6f9e231815bb3c9bc3f0b752398a4ce2276258c7b4be1d466eca3105afc
	Finalized Previous:  Block Height: 2504104, Block Hash: 0x5f2c5191fe8e7b4479746831ff958b86f619cff205f7882cc154f8a146d4a10a
	Best Next:           Block Height: 2504107, Block Hash: 0x5a6e33fccef7e6e55388ddf00c2303b1c1cf2b764f7b0886645fde43643a1f0e
	Best Previous:       Block Height: 2504106, Block Hash: 0x769f455bcfce156a248812af8a81dfcc84018db6b2a852b7296c13a951819103
	Historical Next:     Block Height: 2000000, Block Hash: 0x6831d536cc3d6408a41a1e50d66f4f48c9c2ed5ffc7cfa7505a5f0251365428f
	Historical Previous: Block Height: 1999999, Block Hash: 0xed2db9aa89ee4b5c9ace26fb721bfbe45541d5386b2b0002eabfa3a939de67ed
*/
