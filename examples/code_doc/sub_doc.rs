use avail_rust_client::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Fetching latest finalized block (hash, height)
	let mut sub = Sub::new(client.clone());
	let finalized_height = client.finalized().block_height().await?;
	let next = sub.next().await?;
	assert_eq!(next.height, finalized_height);

	// Fetching block (hash, height) for a specific height
	sub.set_block_height(190010);
	let next = sub.next().await?;
	assert_eq!(next.height, 190010);

	// Fetching previous one
	sub.set_block_height(190010);
	let previous = sub.prev().await?;
	assert_eq!(previous.height, 190009);

	Ok(())
}
