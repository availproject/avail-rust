use avail_rust_client::{prelude::*, subscription::BlockHeaderSub};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// By default it subscribes to finalized block
	let mut sub = BlockHeaderSub::new(client.clone());
	let next_header = sub.next().await?.expect("Should be there");
	let prev_header = sub.prev().await?.expect("Should be there");
	println!("Finalized Next:      Block Height: {}, Block Hash: {:?}", next_header.number, next_header.hash());
	println!("Finalized Previous:  Block Height: {}, Block Hash: {:?}", prev_header.number, prev_header.hash());

	// Best Blocks
	let mut sub = BlockHeaderSub::new(client.clone());
	sub.use_best_block(true);
	let next_header = sub.next().await?.expect("Should be there");
	let prev_header = sub.prev().await?.expect("Should be there");
	println!("Best Next:           Block Height: {}, Block Hash: {:?}", next_header.number, next_header.hash());
	println!("Best Previous:       Block Height: {}, Block Hash: {:?}", prev_header.number, prev_header.hash());

	// Historical Blocks
	let mut sub = BlockHeaderSub::new(client.clone());
	sub.set_block_height(2000000);
	let next_header = sub.next().await?.expect("Should be there");
	let prev_header = sub.prev().await?.expect("Should be there");
	println!("Historical Next:     Block Height: {}, Block Hash: {:?}", next_header.number, next_header.hash());
	println!("Historical Previous: Block Height: {}, Block Hash: {:?}", prev_header.number, prev_header.hash());

	Ok(())
}

/*
	Expected Output:

	Finalized Next:      Block Height: 2504111, Block Hash: 0x1462b84c31b22b99ec580e572d1b0152286c66d34abd2c931145499db4960aff
	Finalized Previous:  Block Height: 2504110, Block Hash: 0x12db525d820f7d8795874ee495f0cb2ce80ff48bc9e2257b36bb96c414790d49
	Best Next:           Block Height: 2504112, Block Hash: 0x36761b71db25da9f527a01c096f4c7117e22fcb1df22653c6c37922b5bfc9f0e
	Best Previous:       Block Height: 2504111, Block Hash: 0x1462b84c31b22b99ec580e572d1b0152286c66d34abd2c931145499db4960aff
	Historical Next:     Block Height: 2000000, Block Hash: 0x6831d536cc3d6408a41a1e50d66f4f48c9c2ed5ffc7cfa7505a5f0251365428f
	Historical Previous: Block Height: 1999999, Block Hash: 0xed2db9aa89ee4b5c9ace26fb721bfbe45541d5386b2b0002eabfa3a939de67ed
*/
