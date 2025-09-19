use avail_rust_client::{error::Error, prelude::*, subscription::BlockHeaderSub};

pub async fn run_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Historical block
	let mut sub = BlockHeaderSub::new(client.clone());
	sub.set_block_height(1908729);

	let header = sub.next().await?.unwrap();
	assert_eq!(header.number, 1908729);

	let header = sub.next().await?.unwrap();
	assert_eq!(header.number, 1908730);

	// Best Block
	let expected = client.best().block_height().await?;
	let mut sub = BlockHeaderSub::new(client.clone());
	sub.set_follow(true);

	let header = sub.next().await?.unwrap();
	assert_eq!(header.number, expected);

	// Finalized Block
	let expected = client.finalized().block_height().await?;
	let mut sub = BlockHeaderSub::new(client);
	sub.set_follow(false);

	let header = sub.next().await?.unwrap();
	assert_eq!(header.number, expected);

	Ok(())
}
