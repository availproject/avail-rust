use avail_rust_client::{block::Block, error::Error, prelude::*};
use avail_rust_core::avail::session::tx::{PurgeKeys, SetKeys};
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
		let block = Block::new(client.clone(), 1811224).extrinsics();

		let submittable = client.tx().session().set_key(
			"0x80c52d4cb7e3f08b72867f94dfd333a69eceeac33182592115329a295d68213c",
			"0xb5e474b9fe49173536aca3ec8f5d6b3bbb8215691466824400fcef78cbbc9ace",
			"0x26fac592a4216ad35dc0960fef4182a34640b4e19781f4dfbe577fa57b145c7d",
			"0xa41af012eb2c05d873869f8d4bc771b7bbc7fc5968ae683f78497ec6b9a32e15",
			Vec::new(),
		);
		let expected_call = SetKeys::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetKeys>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set keys
	{
		let block = Block::new(client.clone(), 209615).extrinsics();

		let submittable = client.tx().session().purge_key();
		let expected_call = PurgeKeys::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<PurgeKeys>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	Ok(())
}
pub async fn event_test() -> Result<(), Error> {
	Ok(())
}
