use avail_rust::error::ClientError;

use super::wait_for_new_era;

pub async fn run() -> Result<(), ClientError> {
	println!("nomination_pools_create");
	create::run().await?;
	println!("nomination_pools_create_with_pool_id");
	create_with_pool_id::run().await?;
	println!("nomination_pools_join");
	join::run().await?;
	println!("nomination_pools_bond_extra");
	bond_extra::run().await?;
	println!("nomination_pools_unbond");
	unbond::run().await?;
	println!("nomination_pools_set_commission");
	set_commission::run().await?;
	println!("nomination_pools_set_metadata");
	set_metadata::run().await?;
	println!("nomination_pools_set_state");
	set_state::run().await?;
	println!("nomination_pools_set_claim_permission");
	set_claim_permission::run().await?;
	println!("nomination_pools_nominate");
	nominate::run().await?;
	println!("nomination_pools_chill");
	chill::run().await?;

	// Wait for new era
	new_era().await?;
	payout_stakers::run().await?;

	println!("nomination_pools_withdraw_unbonded");
	withdraw_unbonded::run().await?;
	println!("nomination_pools_claim_payout");
	claim_payout::run().await?;
	println!("nomination_pools_claim_payout_other");
	claim_payout_other::run().await?;
	println!("nomination_pools_claim_commission");
	claim_commission::run().await?;

	Ok(())
}

mod create {
	use avail_rust::{prelude::*, transactions::NominationPoolsEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Bob")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let amount = SDK::one_avail() * 100_000u128;
		let root = account::account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?; // Bob
		let nominator = account::account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?; // Bob
		let bouncer = account::account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?; // Bob

		let tx = sdk.tx.nomination_pools.create(amount, root, nominator, bouncer);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<NominationPoolsEvents::Created>(), Some(true), "");
		assert_eq!(events.has::<NominationPoolsEvents::Bonded>(), Some(true), "");

		Ok(())
	}
}

mod create_with_pool_id {
	use avail_rust::{prelude::*, transactions::NominationPoolsEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Eve")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let amount = SDK::one_avail() * 100_000u128;
		let root = account::account_id_from_str("5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw")?; // Eve
		let nominator = account::account_id_from_str("5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw")?; // Eve
		let bouncer = account::account_id_from_str("5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw")?; // Eve
		let pool_id = 0;

		let tx = sdk
			.tx
			.nomination_pools
			.create_with_pool_id(amount, root, nominator, bouncer, pool_id);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<NominationPoolsEvents::Created>(), Some(true), "");
		assert_eq!(events.has::<NominationPoolsEvents::Bonded>(), Some(true), "");

		Ok(())
	}
}

mod join {
	use avail_rust::{prelude::*, transactions::NominationPoolsEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Dave")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let amount = SDK::one_avail() * 100_000u128;
		let pool_id = 1;

		let tx = sdk.tx.nomination_pools.join(amount, pool_id);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<NominationPoolsEvents::Bonded>(), Some(true), "");

		Ok(())
	}
}

mod bond_extra {
	use avail_rust::{
		prelude::*,
		transactions::{nom_pools::BondExtra, NominationPoolsEvents},
	};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Dave")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let extra = BondExtra::FreeBalance(SDK::one_avail());

		let tx = sdk.tx.nomination_pools.bond_extra(extra);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<NominationPoolsEvents::Bonded>(), Some(true), "");

		Ok(())
	}
}

mod unbond {
	use avail_rust::{prelude::*, transactions::NominationPoolsEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Dave")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let member_account = account.public_key().to_account_id();
		let unbonding_points = SDK::one_avail();

		let tx = sdk.tx.nomination_pools.unbond(member_account, unbonding_points);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<NominationPoolsEvents::Unbonded>(), Some(true), "");

		Ok(())
	}
}

mod withdraw_unbonded {
	use avail_rust::{prelude::*, transactions::NominationPoolsEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Dave")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let member_account = account.public_key().to_account_id();
		let num_slashing_spans = 0;

		let tx = sdk
			.tx
			.nomination_pools
			.withdraw_unbonded(member_account, num_slashing_spans);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<NominationPoolsEvents::Withdrawn>(), Some(true), "");

		Ok(())
	}
}

mod set_commission {
	use avail_rust::{
		prelude::*,
		transactions::{nom_pools::NewCommission, NominationPoolsEvents},
	};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Bob")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let pool_id = 1;
		let new_commission = NewCommission {
			payee: account::account_id_from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")?, // Alice
			amount: Perbill(10_000_000u32),                                                           // 1%
		};

		let tx = sdk.tx.nomination_pools.set_commission(pool_id, Some(new_commission));
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(
			events.has::<NominationPoolsEvents::PoolCommissionUpdated>(),
			Some(true),
			""
		);

		Ok(())
	}
}

mod set_metadata {
	use avail_rust::prelude::*;
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Bob")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let pool_id = 1;
		let metadata = String::from("This is metadata").as_bytes().to_vec();

		let tx = sdk.tx.nomination_pools.set_metadata(pool_id, metadata);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		Ok(())
	}
}

mod set_state {
	use avail_rust::{
		prelude::*,
		transactions::{nom_pools::State, NominationPoolsEvents},
	};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Eve")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let pool_id = 0;
		let state = State::Destroying;

		let tx = sdk.tx.nomination_pools.set_state(pool_id, state);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<NominationPoolsEvents::StateChanged>(), Some(true), "");

		Ok(())
	}
}

mod set_claim_permission {
	use avail_rust::{prelude::*, transactions::nom_pools::Permission};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Dave")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let permission = Permission::PermissionlessAll;

		let tx = sdk.tx.nomination_pools.set_claim_permission(permission);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		Ok(())
	}
}

mod nominate {
	use avail_rust::{prelude::*, transactions::NominationPoolsCalls};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Bob")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let pool_id = 1;
		let validators = vec![
			account::account_id_from_str("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY")?, // Alice_Stash
		];

		let tx = sdk.tx.nomination_pools.nominate(pool_id, validators);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		assert_eq!(res.is::<NominationPoolsCalls::Nominate>().await.unwrap(), true, "");

		Ok(())
	}
}

mod chill {
	use avail_rust::prelude::*;
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Eve")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let pool_id = 0;

		let tx = sdk.tx.nomination_pools.chill(pool_id);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		Ok(())
	}
}

mod claim_payout {
	use avail_rust::{prelude::*, transactions::NominationPoolsEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Bob")?;
		let account = Keypair::from_uri(&secret_uri)?;

		let tx = sdk.tx.nomination_pools.claim_payout();
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<NominationPoolsEvents::PaidOut>(), Some(true), "");

		Ok(())
	}
}

mod claim_payout_other {
	use avail_rust::{prelude::*, transactions::NominationPoolsEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Bob")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let other = account::account_id_from_str("5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy")?; // Dave

		let tx = sdk.tx.nomination_pools.claim_payout_other(other);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<NominationPoolsEvents::PaidOut>(), Some(true), "");

		Ok(())
	}
}

mod claim_commission {
	use avail_rust::{prelude::*, transactions::NominationPoolsEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Bob")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let pool_id = 1;

		let tx = sdk.tx.nomination_pools.claim_commission(pool_id);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(
			events.has::<NominationPoolsEvents::PoolCommissionClaimed>(),
			Some(true),
			""
		);

		Ok(())
	}
}

mod payout_stakers {
	use avail_rust::prelude::*;
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Alice")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let validator_stash = account::account_id_from_str("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY")?; // Alice Stash
		let era_storage = avail::storage().staking().active_era();
		let storage = sdk.client.storage().at_latest().await?;
		let era = storage.fetch(&era_storage).await?;
		let mut era = era.map(|e| e.index).unwrap_or(0);
		if era > 0 {
			era = era - 1
		};

		let tx = sdk.tx.staking.payout_stakers(validator_stash, era);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		Ok(())
	}
}

async fn new_era() -> Result<(), ClientError> {
	use avail_rust::{avail, SDK};

	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let era_storage = avail::storage().staking().active_era();
	let storage = sdk.client.storage().at_latest().await?;
	let era = storage.fetch(&era_storage).await?;
	let target_era = era.map(|e| e.index).unwrap_or(0) + 3;

	println!("Waiting for era: {}", target_era);

	wait_for_new_era(Some(target_era)).await
}
