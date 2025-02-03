use avail_rust::{account, error::ClientError, SDK};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;

	let alice_account = account::account_id_from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")?;

	let info = account::account_info(&sdk.client, alice_account).await?;
	println!("Flags: {:?}", info.data.flags);
	println!("Free: {}", info.data.free);
	println!("Frozen: {}", info.data.frozen);
	println!("Reserved: {}", info.data.reserved);

	Ok(())
}
