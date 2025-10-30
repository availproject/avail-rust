use avail_rust_client::{prelude::*, subscription::ExtrinsicSub};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// By default it subscribes to finalized block
	// It will return the first block that has at least one extrinsic that we want
	let mut sub = ExtrinsicSub::<avail::data_availability::tx::SubmitData>::new(client.clone(), Default::default());
	let (extrinsics, block_info) = sub.next().await?;
	println!("Finalized:  Block Height: {}, DA Extrinsics Count: {}", block_info.height, extrinsics.len());

	// Best Blocks
	// It will return the first block that has at least one extrinsic that we want
	let mut sub = ExtrinsicSub::<avail::data_availability::tx::SubmitData>::new(client.clone(), Default::default());
	sub.use_best_block(true);
	let (extrinsics, block_info) = sub.next().await?;
	println!("Best:       Block Height: {}, DA Extrinsics Count: {}", block_info.height, extrinsics.len());

	// Historical Blocks
	// It will return the first block that has at least one extrinsic that we want
	let mut sub = ExtrinsicSub::<avail::data_availability::tx::SubmitData>::new(client.clone(), Default::default());
	sub.set_block_height(2000000);
	let (extrinsics, block_info) = sub.next().await?;
	println!("Historical: Block Height: {}, DA Extrinsics Count: {}", block_info.height, extrinsics.len());

	Ok(())
}

/*
	Expected Output:

	Finalized:  Block Height: 2504160, DA Extrinsics Count: 1
	Best:       Block Height: 2504161, DA Extrinsics Count: 4
	Historical: Block Height: 2000001, DA Extrinsics Count: 1
*/
