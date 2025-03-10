/* use avail_rust::error::ClientError;

pub async fn run() -> Result<(), ClientError> {
	println!("session_set_key");
	set_keys::run().await?;

	Ok(())
}

mod set_keys {
	use avail_rust::{prelude::*, transactions::SessionCalls};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Alice//stash")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let keys = rpc::author::rotate_keys(&sdk.client).await?;

		let tx = sdk.tx.session.set_keys(keys);
		let res = tx.execute_and_watch_inclusion(&account, Options::new()).await?;
		assert_eq!(res.is_successful(), Some(true), "Transaction must be successful");
		assert_eq!(res.is::<SessionCalls::SetKeys>().await.unwrap(), true, "");

		Ok(())
	}
}
 */