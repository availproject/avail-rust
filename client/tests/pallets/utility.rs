use avail_rust_client::{block::Block, error::Error, prelude::*};
use avail_rust_core::avail::utility::tx::{Batch, BatchAll, ForceBatch};
use codec::Encode;

pub async fn run_tests() -> Result<(), Error> {
	tx_tests().await?;
	event_test().await?;

	Ok(())
}
pub async fn tx_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Batch
	{
		let block = Block::new(client.clone(), 1828776).signed();

		let c1 = client.tx().staking().chill();
		let c2 = client.tx().staking().unbond(1020000000000000000000);
		let submittable = client.tx().utility().batch(vec![c1, c2]);
		let expected_call = Batch::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Batch>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
		assert_eq!(actual_ext.call.decode_calls().unwrap().len(), expected_call.decode_calls().unwrap().len());
	}

	// Batch All
	{
		let block = Block::new(client.clone(), 1827667).signed();

		let c1 = client.tx().staking().chill();
		let c2 = client.tx().staking().unbond(8371491570236280685776);
		let submittable = client.tx().utility().batch_all(vec![c1, c2]);
		let expected_call = BatchAll::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<BatchAll>(3).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
		assert_eq!(actual_ext.call.decode_calls().unwrap().len(), expected_call.decode_calls().unwrap().len());
	}

	// Force Batch
	{
		let block = Block::new(client.clone(), 1815311).signed();

		let c1 = client
			.tx()
			.staking()
			.payout_stakers("0xb4125a5595f7818337330dc3959ae1bfa3b363be621e6668122abe8dd6f18e0a", 418);
		let c2 = client
			.tx()
			.staking()
			.payout_stakers("0xb4125a5595f7818337330dc3959ae1bfa3b363be621e6668122abe8dd6f18e0a", 419);
		let submittable = client.tx().utility().force_batch(vec![c1, c2]);
		let expected_call = ForceBatch::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<ForceBatch>(4).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
		assert_eq!(actual_ext.call.decode_calls().unwrap().len(), expected_call.decode_calls().unwrap().len());
	}

	Ok(())
}
pub async fn event_test() -> Result<(), Error> {
	Ok(())
}
