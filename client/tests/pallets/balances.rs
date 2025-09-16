use avail_rust_client::{
	block::{BlockEvents, BlockWithTx},
	error::Error,
	prelude::*,
};
use avail_rust_core::avail::balances::{
	events::{Deposit, Endowed, Locked, Reserved, Transfer, Unlocked, Unreserved, Withdraw},
	tx::{TransferAll, TransferAllowDeath, TransferKeepAlive},
};
use codec::Encode;

pub async fn run_tests() -> Result<(), Error> {
	tx_tests().await?;
	event_test().await?;

	Ok(())
}
pub async fn tx_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Transfer All
	{
		let block = BlockWithTx::new(client.clone(), 1828050);

		let account_id = "0x28806db1fa697e9c4967d8bd8ee78a994dfea2887486c39969a7d16bfebbf36f";
		let submittable = client.tx().balances().transfer_all(account_id, false);
		let expected_call = TransferAll::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<TransferAll>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// TransferAllowDeath
	{
		let block = BlockWithTx::new(client.clone(), 1828972);
		let account_id = "0x0d584a4cbbfd9a4878d816512894e65918e54fae13df39a6f520fc90caea2fb0";
		let amount = 2010899374608366600109698;
		let submittable = client.tx().balances().transfer_allow_death(account_id, amount);
		let expected_call = TransferAllowDeath::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<TransferAllowDeath>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// TransferKeepAlive
	{
		let block = BlockWithTx::new(client.clone(), 1828947);
		let account_id = "0x00d6fb2b0c83e1bbf6938265912d900f57c9bee67bd8a8cb18ec50fefbf47931";
		let amount = 616150000000000000000;
		let submittable = client.tx().balances().transfer_keep_alive(account_id, amount);
		let expected_call = TransferKeepAlive::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<TransferKeepAlive>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	Ok(())
}
pub async fn event_test() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	let block = BlockEvents::new(client.clone(), 1861163);
	let events = block.ext(1).await?.unwrap();

	// Withdraw
	let expected = Withdraw {
		amount: 125294783490551801u128,
		who: AccountId::from_str("5GTefZ16Yy5AwMEgeRFDLo6cG3ayy4DVPpFDkjCuvSJJMt3i").unwrap(),
	};
	let actual = events.first::<Withdraw>().unwrap();
	assert_eq!(actual.to_event(), expected.to_event());

	// Endowed
	let expected = Endowed {
		account: AccountId::from_str("0xb96a560df143b2e49f989a4e2c4786e7abf7400c9fe39427d84f83b22a2d4e0b").unwrap(),
		free_balance: 3744383889315788884073,
	};
	let actual = events.first::<Endowed>().unwrap();
	assert_eq!(actual.to_event(), expected.to_event());

	// Transfer
	let expected = Transfer {
		from: AccountId::from_str("0xc270d5832919913ab755e7cc1823811588e8c2f79f8b68e908800014fd96881c").unwrap(),
		to: AccountId::from_str("0xb96a560df143b2e49f989a4e2c4786e7abf7400c9fe39427d84f83b22a2d4e0b").unwrap(),
		amount: 3744383889315788884073,
	};
	let actual = events.first::<Transfer>().unwrap();
	assert_eq!(actual.to_event(), expected.to_event());

	// Deposit
	let deposits = events.all::<Deposit>().unwrap();
	assert_eq!(deposits.len(), 3);
	let expected = Deposit {
		who: AccountId::from_str("0x6d6f646c70792f74727372790000000000000000000000000000000000000000").unwrap(),
		amount: 100235826792441440,
	};
	let actual = &deposits[1];
	assert_eq!(actual.to_event(), expected.to_event());

	// Reserved
	let block = BlockEvents::new(client.clone(), 1861590);
	let events = block.ext(1).await?.unwrap();

	let expected = Reserved {
		who: AccountId::from_str("0x4c4062701850428210b0bb341c92891c2cd8f67c5e66326991f8ee335de2394a").unwrap(),
		amount: 2100000000000000000,
	};
	let actual = events.first::<Reserved>().unwrap();
	assert_eq!(actual.to_event(), expected.to_event());

	// Unreserved
	let block = BlockEvents::new(client.clone(), 1861592);
	let events = block.ext(1).await?.unwrap();

	let expected = Unreserved {
		who: AccountId::from_str("0x4c4062701850428210b0bb341c92891c2cd8f67c5e66326991f8ee335de2394a").unwrap(),
		amount: 2100000000000000000,
	};
	let actual = events.first::<Unreserved>().unwrap();
	assert_eq!(actual.to_event(), expected.to_event());

	// Unlocked
	let block = BlockEvents::new(client.clone(), 1861592);
	let events = block.ext(1).await?.unwrap();

	let expected = Unlocked {
		who: AccountId::from_str("0x248fa9bcba295608e1a3d36455a536ac4e4011e8366d8f56effb732b30dc372b").unwrap(),
		amount: 77000000000000000000000,
	};
	let actual = events.first::<Unlocked>().unwrap();
	assert_eq!(actual.to_event(), expected.to_event());

	// Locked
	let client = Client::new(TURING_ENDPOINT).await?;
	let block = BlockEvents::new(client.clone(), 2280015);
	let events = block.ext(1).await?.unwrap();

	let expected = Locked {
		who: AccountId::from_str("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ").unwrap(),
		amount: 24347340768494881376,
	};
	let actual = events.first::<Locked>().unwrap();
	assert_eq!(actual.to_event(), expected.to_event());

	Ok(())
}
