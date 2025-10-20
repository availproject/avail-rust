use avail_rust_client::{
	block::{Events, SignedExtrinsics},
	error::Error,
	prelude::*,
};
use avail_rust_core::avail::data_availability::{
	events::{ApplicationKeyCreated, DataSubmitted},
	tx::{CreateApplicationKey, SubmitData},
};
use codec::Encode;

pub async fn run_tests() -> Result<(), Error> {
	tx_tests().await?;
	event_test().await?;

	Ok(())
}
pub async fn tx_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// CreateApplicationKey
	{
		let block = SignedExtrinsics::new(client.clone(), 1783406);

		let submittable = client.tx().data_availability().create_application_key("kraken");
		let expected_call = CreateApplicationKey::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<CreateApplicationKey>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Submit Data
	{
		let block = SignedExtrinsics::new(client.clone(), 0);

		let submittable = client
			.tx()
			.data_availability()
			.submit_data("The future is available for all, one block at a time.");
		let expected_call = SubmitData::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SubmitData>(0).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	Ok(())
}
pub async fn event_test() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// ApplicationKeyCreated
	let block = Events::new(client.clone(), 1783406);

	let events = block.extrinsic(1).await.unwrap().unwrap();
	let owner = AccountId::from_str("0x268d78a6783f236eca1e54e8053aa42d8bd138d549e2473c898b482e270f2c56").unwrap();
	let expected = ApplicationKeyCreated { id: 41, key: "kraken".as_bytes().to_vec(), owner };
	let actual = events.first::<ApplicationKeyCreated>().unwrap();
	assert_eq!(actual.to_event(), expected.to_event());

	// DataSubmitted
	let block = Events::new(client.clone(), 1861947);

	let events = block.extrinsic(1).await.unwrap().unwrap();
	let data_hash = H256::from_str("0x04771cf2fabb927e3a3bbbc1096c9ad85d5e3c98ffdc9c26c574e6a079fb3914").unwrap();
	let who = AccountId::from_str("0x6e7b54d8c3a0db834338c6dc3ec02cab9af483e1fdafe24afb0d3d1bd19c0f77").unwrap();
	let expected = DataSubmitted { data_hash, who };
	let actual = events.first::<DataSubmitted>().unwrap();
	assert_eq!(actual.to_event(), expected.to_event());

	Ok(())
}
