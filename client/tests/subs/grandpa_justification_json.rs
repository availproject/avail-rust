use avail_rust_client::{error::Error, prelude::*, subscription::GrandpaJustificationSub};

pub async fn run_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Historical block
	let mut sub = GrandpaJustificationSub::new(client.clone());

	sub.set_block_height(1900032);
	let n = sub.next().await?;
	assert_eq!(n.commit.target_number, 1900032);

	sub.set_block_height(1900122);
	let n = sub.next().await?;
	assert_eq!(n.commit.target_number, 1900122);

	Ok(())
}
