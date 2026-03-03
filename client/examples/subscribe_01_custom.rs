use avail_rust_client::{ext::async_trait, prelude::*};

// Custom fetcher
//
// The subscription system is generic over a Fetcher trait. The built-in fetchers (blocks, headers,
// events, etc.) cover the common cases, but you can implement your own to extract whatever data
// you need from each block.
//
// A Fetcher needs three things:
// - An Output type            - the value produced for each block
// - A fetch() method          - turns a BlockInfo into your Output using the client
// - An optional is_empty()    - tells the subscription when to skip a block (used with .skip_empty())
//
// Your fetcher must also derive Clone.

#[derive(Clone)]
struct ExtrinsicCountFetcher;

#[async_trait]
impl Fetcher for ExtrinsicCountFetcher {
	type Output = usize;

	async fn fetch(&self, client: &Client, info: BlockInfo, _retry: RetryPolicy) -> Result<usize, Error> {
		let count = client
			.block(info.height)
			.extrinsics()
			.count(None, Default::default())
			.await?;
		Ok(count)
	}

	fn is_empty(&self, value: &usize) -> bool {
		*value == 0
	}
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::connect(TURING_ENDPOINT).await?;

	// Once you have a fetcher, plug it into SubscriptionBuilder::new() directly. All the usual
	// builder options (mode, from_height, skip_empty, etc.) work exactly the same way.
	let mut sub = SubscriptionBuilder::new(client, ExtrinsicCountFetcher)
		.from_height(2_000_000)
		.skip_empty()
		.build()
		.await?;

	let item = sub.next().await?;
	println!("Block {}: {} extrinsics", item.block_height, item.value);

	Ok(())
}
