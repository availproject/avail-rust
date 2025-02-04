use avail_rust::error::ClientError;

pub async fn run() -> Result<(), ClientError> {
	println!("balances_transfer_all");
	transfer_all::run().await?;
	transfer_all::clean().await?;
	println!("balances_transfer_allow_death");
	transfer_allow_death::run().await?;
	println!("balances_transfer_keep_alive");
	transfer_keep_alive::run().await?;

	Ok(())
}

mod transfer_all {
	use avail_rust::{
		prelude::*,
		transactions::{BalancesEvents, SystemEvents},
	};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Alice")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let dest = account::account_id_from_str("5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw")?; // Eve
		let keep_alive = false;

		let tx = sdk.tx.balances.transfer_all(dest, keep_alive);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<BalancesEvents::Transfer>(), Some(true), "");
		assert_eq!(events.has::<SystemEvents::KilledAccount>(), Some(true), "");

		Ok(())
	}

	pub async fn clean() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Eve")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let dest = account::account_id_from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")?; // Alice
		let value = SDK::one_avail() * 900_000;

		let tx = sdk.tx.balances.transfer_keep_alive(dest, value);
		tx.execute_and_watch_inclusion(&account, None).await?;

		Ok(())
	}
}

mod transfer_allow_death {
	use avail_rust::{prelude::*, transactions::BalancesEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Alice")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let dest = account::account_id_from_str("5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw")?; // Eve
		let amount = SDK::one_avail();

		let tx = sdk.tx.balances.transfer_allow_death(dest, amount);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<BalancesEvents::Transfer>(), Some(true), "");

		Ok(())
	}
}

mod transfer_keep_alive {
	use avail_rust::{prelude::*, transactions::BalancesEvents};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Alice")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let dest = account::account_id_from_str("5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw")?; // Eve
		let amount = SDK::one_avail();

		let tx = sdk.tx.balances.transfer_keep_alive(dest, amount);
		let res = tx.execute_and_watch_inclusion(&account, None).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");

		let events = res.events.unwrap();
		assert_eq!(events.has::<BalancesEvents::Transfer>(), Some(true), "");

		Ok(())
	}
}
