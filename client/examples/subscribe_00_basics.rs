use avail_rust_client::prelude::*;
use avail_rust_core::rpc::AllowedEvents;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::connect(TURING_ENDPOINT).await?;

	// Subscribing to blocks
	//
	// The simplest way to subscribe is via client.subscribe().blocks().build(). This gives us a
	// Subscription that follows finalized blocks starting from the current chain head. Each call
	// to .next() waits for the next block and returns a SubscriptionItem containing the value,
	// block height, and block hash.
	let mut sub = client.subscribe().blocks().build().await?;
	let block = sub.next().await?;
	println!("Height: {}, Hash: {}", block.block_height, block.block_hash);

	// The builder exposes a few knobs before calling .build():
	//
	// .mode(BlockQueryMode::Best)   - follow best (non-finalized) blocks instead of finalized
	// .from_height(N)               - start from a specific historical block height
	// .poll_interval(Duration)      - how often to poll for new blocks (default: 3s)
	// .retry(RetryPolicy)           - retry policy for failed RPC calls
	// .skip_empty()                 - skip blocks where the fetcher returns empty data
	//
	// By default the subscription follows finalized blocks from the current height.
	let mut sub = client.subscribe().blocks().mode(BlockQueryMode::Best).build().await?;
	let _ = sub.next().await?;

	let mut sub = client.subscribe().blocks().from_height(2_000_000).build().await?;
	let _ = sub.next().await?;

	// Bidirectional navigation
	//
	// Subscriptions support both .next() and .prev() for walking the chain in either direction.
	// This is handy when you need to look back at the block you just processed or replay a range
	// in reverse.
	let mut sub = client.subscribe().blocks().from_height(2_000_000).build().await?;
	let current = sub.next().await?;
	println!("Current: {}", current.block_height);

	let previous = sub.prev().await?;
	println!("Previous: {}", previous.block_height);

	// Stream conversion
	//
	// If you prefer an async Stream over manual .next() calls, .into_stream() converts the
	// subscription into a standard futures::Stream. This plays nicely with StreamExt combinators
	// like .take(), .filter(), .for_each(), etc.
	let sub = client.subscribe().blocks().build().await?;
	let mut stream = std::pin::pin!(sub.into_stream());

	use futures::StreamExt;
	if let Some(Ok(block)) = stream.next().await {
		println!("Stream: {}", block.block_height);
	}

	// Data types
	//
	// The subscribe API supports several fetchers that control what data each block yields.
	// All of them go through the same builder pattern, so the options above (mode, from_height,
	// skip_empty, etc.) work with every fetcher.
	//
	// .raw()             - just block height and hash, no extra RPC calls
	// .blocks()          - a lazy Block handle for each block
	// .block_headers()   - the block header only
	// .legacy_blocks()   - full legacy block with justification
	// .block_events(..)  - filtered events within a block
	// .extrinsics(..)    - decoded extrinsics of a specific type
	// .justification()   - GRANDPA justifications
	let mut sub = client.subscribe().raw().build().await?;
	let _ = sub.next().await?;

	let mut sub = client.subscribe().block_headers().build().await?;
	let _ = sub.next().await?;

	let mut sub = client.subscribe().legacy_blocks().build().await?;
	let _ = sub.next().await?;

	let mut sub = client.subscribe().justification().build().await?;
	let _ = sub.next().await?;

	// For block_events we pass an AllowedEvents filter and typically skip blocks with no matching
	// events. The .skip_empty() builder option is especially useful here.
	let mut sub = client
		.subscribe()
		.block_events(AllowedEvents::OnlyExtrinsics)
		.skip_empty()
		.build()
		.await?;
	let events = sub.next().await?;
	println!("Events: height={}, count={}", events.block_height, events.value.len());

	Ok(())
}
