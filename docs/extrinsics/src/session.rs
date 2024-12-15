use avail_rust::error::ClientError;

pub async fn run() -> Result<(), ClientError> {
	println!("session_set_key");
	set_keys::run().await?;

	Ok(())
}

mod set_keys {
	use avail_rust::{
		error::ClientError, rpc, transactions::SessionCalls, utils, Keypair, SecretUri, SDK,
	};
	use core::str::FromStr;

	pub async fn run() -> Result<(), ClientError> {
		let sdk = SDK::new(SDK::local_endpoint()).await?;

		// Input
		let secret_uri = SecretUri::from_str("//Alice//stash")?;
		let account = Keypair::from_uri(&secret_uri)?;
		let keys = rpc::author::rotate_keys(&sdk.rpc_client).await?;
		let keys = utils::deconstruct_session_keys(keys)?;

		let tx = sdk.tx.session.set_keys(keys);
		let result = tx.execute_and_watch_inclusion(&account, None).await?;

		result.print_debug();
		if let Some(data) = result
			.get_data::<SessionCalls::SetKeys>(&sdk.online_client)
			.await
		{
			dbg!(data);
		}

		Ok(())
	}
}
