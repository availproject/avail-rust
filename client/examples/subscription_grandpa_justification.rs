use avail_rust_client::{prelude::*, subscription::GrandpaJustificationSub};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// By default it subscribes to finalized block
	let mut sub = GrandpaJustificationSub::new(client.clone());
	let (justification, block_info) = sub.next().await?;
	println!("Block {} has grandpa justifications: {}", block_info.height, justification.is_some());

	// Historical Blocks
	let mut sub = GrandpaJustificationSub::new(client.clone());
	sub.set_block_height(2000384);
	let (justification, block_info) = sub.next().await?;
	println!("Block {} has grandpa justifications: {}", block_info.height, justification.is_some());

	Ok(())
}

/*
	Expected Output:

	Block 2504246 has grandpa justifications: false
	Block 2000384 has grandpa justifications: true
*/
