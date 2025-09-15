use avail_rust_client::{
	block::{BlockEvents, BlockWithExt},
	error::Error,
	prelude::*,
};
use avail_rust_core::avail::data_availability::tx::{CreateApplicationKey, SubmitData};
use codec::Encode;

pub async fn run_tests() -> Result<(), Error> {
	tx_tests().await?;
	event_test().await?;

	Ok(())
}
pub async fn tx_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Submit Data
	let block = BlockWithExt::new(client.clone(), 0);
	let data: Vec<u8> = "The future is available for all, one block at a time."
		.bytes()
		.collect();
	let submittable = client.tx().data_availability().submit_data(data);
	let expected_call = SubmitData::from_call(&submittable.call.encode()).unwrap();
	let actual_ext = block.get::<SubmitData>(0).await?.unwrap();
	assert_eq!(actual_ext.call.encode(), expected_call.encode());

	// CreateApplicationKey
	let block = BlockWithExt::new(client.clone(), 1783406);
	let data: Vec<u8> = "kraken".bytes().collect();
	let submittable = client.tx().data_availability().create_application_key(data);
	let expected_call = CreateApplicationKey::from_call(&submittable.call.encode()).unwrap();
	let actual_ext = block.get::<CreateApplicationKey>(1).await?.unwrap();
	assert_eq!(actual_ext.call.encode(), expected_call.encode());

	Ok(())
}
pub async fn event_test() -> Result<(), Error> {
	Ok(())
}
