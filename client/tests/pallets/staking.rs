use avail_rust_client::{block::Block, error::Error, prelude::*};
use avail_rust_core::avail::staking::{
	events::{Bonded, Chilled, PayoutStarted, Rewarded, Unbonded, ValidatorPrefsSet, Withdraw},
	tx::{
		Bond, BondExtra, Chill, Kick, Nominate, PayoutStakers, PayoutStakersByPage, Rebond, SetController, SetPayee,
		Unbond, Validate, WithdrawUnbonded,
	},
	types::{RewardDestination, ValidatorPrefs},
};
use codec::Encode;

pub async fn run_tests() -> Result<(), Error> {
	tx_tests().await?;
	event_test().await?;

	Ok(())
}
pub async fn tx_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Bond
	{
		let block = Block::new(client.clone(), 1688315).signed();

		let submittable = client
			.tx()
			.staking()
			.bond(50100000000000000000000, RewardDestination::Staked);
		let expected_call = Bond::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Bond>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Bond Extra
	{
		let block = Block::new(client.clone(), 1828569).signed();

		let submittable = client.tx().staking().bond_extra(10000000000000000000);
		let expected_call = BondExtra::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<BondExtra>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Chill
	{
		let block = Block::new(client.clone(), 1811904).signed();

		let submittable = client.tx().staking().chill();
		let expected_call = Chill::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Chill>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// WithdrawUnbonded
	{
		let block = Block::new(client.clone(), 1827511).signed();

		let submittable = client.tx().staking().withdraw_unbonded(84);
		let expected_call = WithdrawUnbonded::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<WithdrawUnbonded>(3).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Validate
	{
		let block = Block::new(client.clone(), 1814105).signed();

		let submittable = client.tx().staking().validate(100000000, false);
		let expected_call = Validate::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Validate>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Unbond
	{
		let block = Block::new(client.clone(), 1827480).signed();

		let submittable = client.tx().staking().unbond(49990000000000000000000);
		let expected_call = Unbond::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Unbond>(4).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// SetPayee
	{
		let block = Block::new(client.clone(), 1785389).signed();

		let account_id =
			AccountId::from_str("0xdc38c8b63df616b7b9662544382c240f5f1c8eb47bc510b6077bd57fba077a5d").unwrap();
		let submittable = client.tx().staking().set_payee(RewardDestination::Account(account_id));
		let expected_call = SetPayee::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetPayee>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Rebond
	{
		let block = Block::new(client.clone(), 1817341).signed();

		let submittable = client.tx().staking().rebond(2134432193417643036990);
		let expected_call = Rebond::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Rebond>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// PayoutStakersByPage
	{
		let block = Block::new(client.clone(), 1807526).signed();

		let address = "0x37dfeeed435f0e9f205e1dfc55775fcd06518f63a5b1ccd53ce2d9e14ab783d3";
		let submittable = client.tx().staking().payout_stakers_by_page(address, 417, 0);
		let expected_call = PayoutStakersByPage::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<PayoutStakersByPage>(2).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// PayoutStakers
	{
		let block = Block::new(client.clone(), 1827501).signed();

		let address = "0xa4605eebf32be28f4b30219a329d5f61d1b250c2780ca62f1875e84adeac8b42";
		let submittable = client.tx().staking().payout_stakers(address, 422);
		let expected_call = PayoutStakers::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<PayoutStakers>(6).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Nominate
	{
		let block = Block::new(client.clone(), 1811815).signed();

		let targets = vec![
			"0x946a8565423df55a0449eb3502f1fff00158aa87aad880ff4a6cab915f2c0058",
			"0x248fa9bcba295608e1a3d36455a536ac4e4011e8366d8f56effb732b30dc372b",
			"0x9a75097e60376fa2c86e6f0830f58be57bf46e3832c5a5b763f4b8a89906483a",
			"0x1ca7f1e157baa7620d46102affe26a6f8322ff1743c80d0a21022f3ef29d0537",
		];
		let submittable = client.tx().staking().nominate(targets);
		let expected_call = Nominate::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Nominate>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Kick
	{
		let block = Block::new(client.clone(), 669361).signed();

		let address = MultiAddress::Address32(
			const_hex::decode("0x64c63961305e9ce5c8d9c43f0db12c141ed6ad25437ed3835c4e6ceab7307d79")
				.unwrap()
				.try_into()
				.unwrap(),
		);
		let submittable = client.tx().staking().kick(vec![address]);
		let expected_call = Kick::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Kick>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set Controller
	{
		let block = Block::new(client.clone(), 470124).signed();

		let submittable = client.tx().staking().set_controller();
		let expected_call = SetController::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetController>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	Ok(())
}
pub async fn event_test() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Bond
	{
		let client = Client::new(TURING_ENDPOINT).await?;
		let events = block::BlockEventsQuery::new(client.clone(), 2280015)
			.extrinsic(1)
			.await
			.unwrap();

		let expected = Bonded {
			stash: AccountId::from_str("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ").unwrap(),
			amount: 24347340768494881376,
		};
		let actual = events.first::<Bonded>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// Unbond
	{
		let events = block::BlockEventsQuery::new(client.clone(), 1835193)
			.extrinsic(1)
			.await
			.unwrap();

		let expected = Unbonded {
			stash: AccountId::from_str("0x7e1180729a6eebfa4c3b2f6cf2f6c7bf4c09f10f3dc339c6de8e1c14c539e62d").unwrap(),
			amount: 87000000000000000000000,
		};
		let actual = events.first::<Unbonded>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// ValidatorPrefsSet
	{
		let events = block::BlockEventsQuery::new(client.clone(), 1814105)
			.extrinsic(1)
			.await
			.unwrap();

		let expected = ValidatorPrefsSet {
			stash: AccountId::from_str("0xbaaf2475c394b0ab52a41966f1668950b4c896fbc365780d13f616bc7577fe3e").unwrap(),
			prefs: ValidatorPrefs { blocked: false, commission: 100000000 },
		};
		let actual = events.first::<ValidatorPrefsSet>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// Chilled
	{
		let events = block::BlockEventsQuery::new(client.clone(), 1811904)
			.extrinsic(1)
			.await
			.unwrap();

		let expected = Chilled {
			stash: AccountId::from_str("0xf2e800a72aa7b4e617f4f4a3f1fd3f02e92d1162049b9000de27d949f5d47c12").unwrap(),
		};
		let actual = events.first::<Chilled>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// Rewarded
	{
		let events = block::BlockEventsQuery::new(client.clone(), 1861532)
			.extrinsic(1)
			.await
			.unwrap();

		let expected = Rewarded {
			stash: AccountId::from_str("0x46fc4b4c46aa309f06f432e69e8447abfafcd083df55727d45cc0c8cfe40543e").unwrap(),
			dest: RewardDestination::Stash,
			amount: 1631460583448789025116,
		};
		let actual = events.first::<Rewarded>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// PayoutStarted
	{
		let events = block::BlockEventsQuery::new(client.clone(), 1861532)
			.extrinsic(1)
			.await
			.unwrap();

		let expected = PayoutStarted {
			era_index: 430,
			validator_stash: AccountId::from_str("0x46fc4b4c46aa309f06f432e69e8447abfafcd083df55727d45cc0c8cfe40543e")
				.unwrap(),
		};
		let actual = events.first::<PayoutStarted>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// Withdrawn
	{
		let events = block::BlockEventsQuery::new(client.clone(), 1861093)
			.extrinsic(1)
			.await
			.unwrap();

		let expected = Withdraw {
			stash: AccountId::from_str("0xc270d5832919913ab755e7cc1823811588e8c2f79f8b68e908800014fd96881c").unwrap(),
			amount: 3740409175720722019688,
		};
		let actual = events.first::<Withdraw>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	Ok(())
}
