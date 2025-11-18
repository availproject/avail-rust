use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Block Event Count
	let event_count = client.chain().block_event_count(2000000).await?;
	println!("Block 2000000 Event Count: {}", event_count);

	Ok(())
}

/*
	Expected Output:

	Block 2000000 Event Count: 10
*/
