use avail_rust_client::{prelude::*, subscription::BlockEventsSub};
use avail_rust_core::rpc::{EventFilter, EventOpts};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let opts = EventOpts::new().filter(EventFilter::OnlyExtrinsics);

	// By default it subscribes to finalized block
	let mut sub = BlockEventsSub::new(client.clone(), opts.clone());
	let events = sub.next().await?;
	println!("Finalized:  Extrinsic Event Count: {}", events.len());

	// Best Blocks
	let mut sub = BlockEventsSub::new(client.clone(), opts.clone());
	sub.use_best_block(true);
	let events = sub.next().await?;
	println!("Best:       Extrinsic Event Count: {}", events.len());

	// Historical Blocks
	// For some older blocks this will not work as at that time the necessary runtime api was not available
	let mut sub = BlockEventsSub::new(client.clone(), opts.clone());
	sub.set_block_height(2100000);
	let events = sub.next().await?;
	println!("Historical: Extrinsic Event Count: {}", events.len());

	Ok(())
}

/*
	Expected Output:

	Finalized:  Extrinsic Event Count: 3
	Best:       Extrinsic Event Count: 3
	Historical: Extrinsic Event Count: 2
*/
