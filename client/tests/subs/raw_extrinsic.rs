use avail_rust_client::{block::BlockExtOptionsExpanded, error::Error, prelude::*, subscription::RawExtrinsicSub};

pub async fn run_tests() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Historical block #1
	let opts = BlockExtOptionsExpanded { filter: Some((29u8, 1u8).into()), ..Default::default() };
	let mut sub = RawExtrinsicSub::new(client.clone(), opts);

	sub.set_block_height(2326672);
	let (list, info) = sub.next().await?;
	assert_eq!(info.height, 2326672);
	assert_eq!(list.len(), 1);

	let (list, info) = sub.next().await?;
	assert_eq!(info.height, 2326674);
	assert_eq!(list.len(), 1);

	// Historical block #2
	let mut sub = RawExtrinsicSub::new(client.clone(), Default::default());

	sub.set_block_height(2326672);
	let (list, info) = sub.next().await?;
	assert_eq!(info.height, 2326672);
	assert_eq!(list.len(), 3);

	let (list, info) = sub.next().await?;
	assert_eq!(info.height, 2326673);
	assert_eq!(list.len(), 2);

	Ok(())
}
