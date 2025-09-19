use avail_rust_client::{error::Error, prelude::*, subscription::TransactionSub};
use avail_rust_core::avail::data_availability::tx::SubmitData;

pub async fn run_tests() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Historical block
	let mut sub = TransactionSub::<SubmitData>::new(client.clone(), Default::default());

	sub.set_block_height(2326672);
	let (list, info) = sub.next().await?;
	assert_eq!(info.height, 2326672);
	assert_eq!(list.len(), 1);

	let (list, info) = sub.next().await?;
	assert_eq!(info.height, 2326674);
	assert_eq!(list.len(), 1);

	Ok(())
}
