use avail_rust_client::{block::extrinsic_options::Options, prelude::*, subscription::EncodedExtrinsicSub};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let opts = Options::new().filter((29, 1));

	// By default it subscribes to finalized block
	// It will return the first block that has at least one extrinsic that we want
	let mut sub = EncodedExtrinsicSub::new(client.clone(), opts.clone());
	let extrinsics = sub.next().await?;
	println!(
		"Finalized:  Block Height: {}, DA Extrinsics Count: {}",
		extrinsics.block_height,
		extrinsics.list.len()
	);

	// Best Blocks
	// It will return the first block that has at least one extrinsic that we want
	let mut sub = EncodedExtrinsicSub::new(client.clone(), opts.clone());
	sub.use_best_block(true);
	let extrinsics = sub.next().await?;
	println!(
		"Best:       Block Height: {}, DA Extrinsics Count: {}",
		extrinsics.block_height,
		extrinsics.list.len()
	);

	// Historical Blocks
	// It will return the first block that has at least one extrinsic that we want
	let mut sub = EncodedExtrinsicSub::new(client.clone(), opts);
	sub.set_block_height(2000000);
	let extrinsics = sub.next().await?;
	println!(
		"Historical: Block Height: {}, DA Extrinsics Count: {}",
		extrinsics.block_height,
		extrinsics.list.len()
	);

	Ok(())
}

/*
	Expected Output:

	Finalized:  Block Height: 2504172, DA Extrinsics Count: 4
	Best:       Block Height: 2504174, DA Extrinsics Count: 2
	Historical: Block Height: 2000001, DA Extrinsics Count: 1
*/
