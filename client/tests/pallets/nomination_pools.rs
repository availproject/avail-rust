use avail_rust_client::{block_api::BlockWithTx, error::Error, prelude::*};
use avail_rust_core::avail::nomination_pools::{
	tx::{
		BondExtra, BondExtraOther, Chill, ClaimCommission, ClaimPayout, ClaimPayoutOther, Create, CreateWithPoolId,
		Join, Nominate, SetClaimPermission, SetCommission, SetCommissionChangeRate, SetCommissionMax, SetMetadata,
		SetState, Unbond, UpdateRoles, WithdrawUnbonded,
	},
	types::{BondExtraValue, ClaimPermission, ConfigOpAccount, PoolState},
};
use codec::Encode;

pub async fn run_tests() -> Result<(), Error> {
	tx_tests().await?;
	event_test().await?;

	Ok(())
}
pub async fn tx_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Bond Extra
	{
		let block = BlockWithTx::new(client.clone(), 1831776);

		let submittable = client.tx().nomination_pools().bond_extra(BondExtraValue::Rewards);
		let expected_call = BondExtra::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<BondExtra>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Bond Extra #2
	{
		let block = BlockWithTx::new(client.clone(), 1831566);

		let submittable = client
			.tx()
			.nomination_pools()
			.bond_extra(BondExtraValue::FreBalance(6740000000000000000));
		let expected_call = BondExtra::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<BondExtra>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Bond Extra Other
	{
		let block = BlockWithTx::new(client.clone(), 202579);

		let submittable = client.tx().nomination_pools().bond_extra_other(
			"0xe48387e8f162d580110568e3df575054de32269822f2362702a8afb1f6914469",
			BondExtraValue::Rewards,
		);
		let expected_call = BondExtraOther::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<BondExtraOther>(2).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Chill
	{
		let block = BlockWithTx::new(client.clone(), 1729911);

		let submittable = client.tx().nomination_pools().chill(15);
		let expected_call = Chill::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Chill>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Claim Commission
	{
		let block = BlockWithTx::new(client.clone(), 1802972);

		let submittable = client.tx().nomination_pools().claim_commission(78);
		let expected_call = ClaimCommission::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<ClaimCommission>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Claim Payout
	{
		let block = BlockWithTx::new(client.clone(), 1831831);

		let submittable = client.tx().nomination_pools().claim_payout();
		let expected_call = ClaimPayout::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<ClaimPayout>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Claim Payout Other
	{
		let block = BlockWithTx::new(client.clone(), 535568);

		let submittable = client
			.tx()
			.nomination_pools()
			.claim_payout_other("0x7e1180729a6eebfa4c3b2f6cf2f6c7bf4c09f10f3dc339c6de8e1c14c539e62d");
		let expected_call = ClaimPayoutOther::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<ClaimPayoutOther>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Create
	{
		let block = BlockWithTx::new(client.clone(), 182681);

		let address = "0x80acee285f2fd1b1042690b2e4447eac328fe6f70d32badd9ffbba4c872a6319";
		let submittable = client
			.tx()
			.nomination_pools()
			.create(10000000000000000000000, address, address, address);
		let expected_call = Create::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Create>(14).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Create With Pool Id
	{
		let block = BlockWithTx::new(client.clone(), 481224);

		let address = "0xc2ff325a289cf3c42e9ab0af62f285a22e8ec6ce0498c50318b5e6d4da827653";
		let submittable =
			client
				.tx()
				.nomination_pools()
				.create_with_pool_id(10000000000000000000000, address, address, address, 37);
		let expected_call = CreateWithPoolId::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<CreateWithPoolId>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Join
	{
		let block = BlockWithTx::new(client.clone(), 1822288);

		let submittable = client.tx().nomination_pools().join(365000000000000000000, 4);
		let expected_call = Join::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Join>(2).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Nominate
	{
		let block = BlockWithTx::new(client.clone(), 1808990);

		let validators = vec![
			"0xa26556769ad6581b7beb103590a5c378955244aa349bbacc2f148c51205e055a",
			"0xa586680015c5b7fe08486de7ba5a8e2064dea3324ecaeda658f3b5443d37c5c1",
		];
		let submittable = client.tx().nomination_pools().nominate(50, validators);
		let expected_call = Nominate::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Nominate>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set Claim Permission #1
	{
		let block = BlockWithTx::new(client.clone(), 1827335);

		let submittable = client
			.tx()
			.nomination_pools()
			.set_claim_permission(ClaimPermission::Permissioned);
		let expected_call = SetClaimPermission::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetClaimPermission>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set Claim Permission #2
	{
		let block = BlockWithTx::new(client.clone(), 1827272);

		let submittable = client
			.tx()
			.nomination_pools()
			.set_claim_permission(ClaimPermission::PermissionlessCompound);
		let expected_call = SetClaimPermission::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetClaimPermission>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set Claim Permission #3
	{
		let block = BlockWithTx::new(client.clone(), 1716287);

		let submittable = client
			.tx()
			.nomination_pools()
			.set_claim_permission(ClaimPermission::PermissionlessAll);
		let expected_call = SetClaimPermission::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetClaimPermission>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set Commission #1
	{
		let block = BlockWithTx::new(client.clone(), 1181206);

		let account_id: AccountId =
			AccountId::from_str("0xec5c245a8405d77710d5d226e354b4236e5e5d13c61fa8ba3fa9aed204b6d6b7").unwrap();
		let submittable = client
			.tx()
			.nomination_pools()
			.set_commission(73, Some((10000000, account_id.into())));
		let expected_call = SetCommission::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetCommission>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set Commission #2
	{
		let block = BlockWithTx::new(client.clone(), 1056874);

		let submittable = client.tx().nomination_pools().set_commission(76, None);
		let expected_call = SetCommission::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetCommission>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set Commission Change Rate
	{
		let block = BlockWithTx::new(client.clone(), 493706);

		let submittable = client
			.tx()
			.nomination_pools()
			.set_commission_change_rate(76, 1000000000, 4320);
		let expected_call = SetCommissionChangeRate::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetCommissionChangeRate>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set Commission Max
	{
		let block = BlockWithTx::new(client.clone(), 472501);

		let submittable = client.tx().nomination_pools().set_commission_max(76, 100000000);
		let expected_call = SetCommissionMax::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetCommissionMax>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set Metadata
	{
		let block = BlockWithTx::new(client.clone(), 182911);

		let submittable = client.tx().nomination_pools().set_metadata(78, "Green");
		let expected_call = SetMetadata::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetMetadata>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set State #1
	{
		let block = BlockWithTx::new(client.clone(), 86141);

		let submittable = client.tx().nomination_pools().set_state(37, PoolState::Destroying);
		let expected_call = SetState::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetState>(4).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Set State #2
	{
		let block = BlockWithTx::new(client.clone(), 337747);

		let submittable = client.tx().nomination_pools().set_state(55, PoolState::Blocked);
		let expected_call = SetState::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<SetState>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Unbond
	{
		let block = BlockWithTx::new(client.clone(), 1831014);

		let member_account = "0xc25a201b2443dac9697558458ccb6b120c079f70b9a72eeeea7914639197e24f";
		let submittable = client
			.tx()
			.nomination_pools()
			.unbond(member_account, 333000000000000000000);
		let expected_call = Unbond::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Unbond>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Update Roles #1
	{
		let block = BlockWithTx::new(client.clone(), 694694);

		let submittable = client.tx().nomination_pools().update_roles(
			29,
			ConfigOpAccount::Remove,
			ConfigOpAccount::Remove,
			ConfigOpAccount::Remove,
		);
		let expected_call = UpdateRoles::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<UpdateRoles>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Update Roles #2
	{
		let block = BlockWithTx::new(client.clone(), 183031);

		let set = ConfigOpAccount::Set(
			AccountId::from_str("0x7b70773cac7dc43f72f79fff8718606f5d2a38077326d9bd1e5c6ac1b1d79fd9").unwrap(),
		);
		let submittable = client
			.tx()
			.nomination_pools()
			.update_roles(68, set.clone(), set.clone(), set);
		let expected_call = UpdateRoles::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<UpdateRoles>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Withdraw Unbonded
	{
		let block = BlockWithTx::new(client.clone(), 1832868);

		let submittable = client
			.tx()
			.nomination_pools()
			.withdraw_unbonded("0x48498d4fdb57d0c11c8e4ec98ffc0a7511563eb73cd2940c5208fc9170bed473", 0);
		let expected_call = WithdrawUnbonded::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<WithdrawUnbonded>(1).await?.unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	Ok(())
}
pub async fn event_test() -> Result<(), Error> {
	Ok(())
}
