use avail_rust_client::{prelude::*, subscription::GrandpaJustificationSub};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// By default it subscribes to finalized block
	let mut sub = GrandpaJustificationSub::new(client.clone());
	let next_just = sub.next().await?;
	let prev_just = sub.prev().await?;
	println!("Block {} has grandpa justifications: {}", next_just.block_height, next_just.value.is_some());
	println!("Block {} has grandpa justifications: {}", prev_just.block_height, prev_just.value.is_some());

	// Historical Blocks
	let mut sub = GrandpaJustificationSub::new(client.clone());
	sub.set_block_height(2000384);
	let just = sub.next().await?;
	println!("Block {} has grandpa justifications: {}", just.block_height, just.value.is_some());

	Ok(())
}

/*
	Expected Output:

	Block 2507990 has grandpa justifications: false
	Block 2507989 has grandpa justifications: false
	Block 2000384 has grandpa justifications: true
*/
