use avail_rust_client::{prelude::*, subscription::LegacyBlockSub};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// By default it subscribes to finalized block
	let mut sub = LegacyBlockSub::new(client.clone());
	let next_block = sub.next().await?.expect("Should be there");
	let prev_block = sub.prev().await?.expect("Should be there");
	println!(
		"Finalized Next:      Block Height: {}, Block Extrinsic Count: {}",
		next_block.block.header.number,
		next_block.block.extrinsics.len(),
	);
	println!(
		"Finalized Previous:  Block Height: {}, Block Extrinsic Count: {}",
		prev_block.block.header.number,
		prev_block.block.extrinsics.len(),
	);

	// Best Blocks
	let mut sub = LegacyBlockSub::new(client.clone());
	sub.use_best_block(true);
	let next_block = sub.next().await?.expect("Should be there");
	let prev_block = sub.prev().await?.expect("Should be there");
	println!(
		"Best Next:           Block Height: {}, Block Extrinsic Count: {}",
		next_block.block.header.number,
		next_block.block.extrinsics.len(),
	);
	println!(
		"Best Previous:       Block Height: {}, Block Extrinsic Count: {}",
		prev_block.block.header.number,
		prev_block.block.extrinsics.len(),
	);

	// Historical Blocks
	let mut sub = LegacyBlockSub::new(client.clone());
	sub.set_block_height(2000000);
	let next_block = sub.next().await?.expect("Should be there");
	let prev_block = sub.prev().await?.expect("Should be there");
	println!(
		"Historical Next:     Block Height: {}, Block Extrinsic Count: {}",
		next_block.block.header.number,
		next_block.block.extrinsics.len(),
	);
	println!(
		"Historical Previous: Block Height: {}, Block Extrinsic Count: {}",
		prev_block.block.header.number,
		prev_block.block.extrinsics.len(),
	);

	Ok(())
}

/*
	Expected Output:

	Finalized Next:      Block Height: 2504149, Block Extrinsic Count: 4
	Finalized Previous:  Block Height: 2504148, Block Extrinsic Count: 3
	Best Next:           Block Height: 2504151, Block Extrinsic Count: 3
	Best Previous:       Block Height: 2504150, Block Extrinsic Count: 8
	Historical Next:     Block Height: 2000000, Block Extrinsic Count: 3
	Historical Previous: Block Height: 1999999, Block Extrinsic Count: 3
*/
