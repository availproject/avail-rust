use avail_rust_client::{block::Block, error::Error, prelude::*};
use avail_rust_core::avail::timestamp::tx::Set;
use codec::Encode;

pub async fn run_tests() -> Result<(), Error> {
	tx_tests().await?;
	event_test().await?;

	Ok(())
}
pub async fn tx_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Set keys
	{
		let block = Block::new(client.clone(), 1896556).extrinsics();

		let expected_call = Set { now: 1758027000000 };
		let actual_ext = block.get::<Set>(0).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	Ok(())
}
pub async fn event_test() -> Result<(), Error> {
	Ok(())
}
