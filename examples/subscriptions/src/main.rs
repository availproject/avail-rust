use avail_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	//
	// Essential Subscription
	//

	// `Sub` is the atomic building block for all other subs
	//
	// By default it:
	// - Follows finalized blocks (use `use_best_block(true)` for best blocks)
	// - Starts from the latest block (or pick one with `set_block_height(h)`)
	// - Retries on RPC errors and polls every 3s (customize with `set_retry_on_error` / `set_pool_rate`)
	//
	// Usage:
	// - `next()` → move forward to the next block `(hash, height)`
	// - `prev()` → move backward to the previous block `(hash, height)`
	let mut sub = Sub::new(client.clone());

	// Setting block height to 190010 will fetch block (hash, height) for that height
	sub.set_block_height(190010);

	// Next moves us forward or fetches the current block (hash, height) if no block (hash, height) was fetched yet
	let info = sub.next().await?;
	assert_eq!(info.height, 190010);
	println!("Block Height: {}, Block Hash: {:?}", info.height, info.hash);

	for i in 1..3 {
		let info = sub.next().await?;
		assert_eq!(info.height, 190010 + i);
	}

	// Prev moves us to the previous block.
	sub.set_block_height(190010);
	let info = sub.prev().await?;
	assert_eq!(info.height, 190010 - 1);

	for i in 1..3 {
		let info = sub.prev().await?;
		assert_eq!(info.height, 190010 - i - 1);
	}

	// If nothing is set then calling `sub.next()` will fetch the latest finalized block (hash, height)
	let mut sub = Sub::new(client.clone());

	let finalized_height = client.finalized().block_height().await?;
	let next = sub.next().await?;
	assert_eq!(next.height, finalized_height);

	// If `sub.use_best_block(true);` is called (before `next` or `prev`)
	// then calling `sub.next()` will fetch the latest best block (hash, height)
	let mut sub = Sub::new(client.clone());
	sub.use_best_block(true);

	let best_height = client.best().block_height().await?;
	let next = sub.next().await?;
	assert_eq!(next.height, best_height);

	Ok(())
}
