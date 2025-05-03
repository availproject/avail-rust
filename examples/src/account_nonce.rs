use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;

	let alice_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

	// Fetch nonce via RPC
	let nonce = account::nonce(&sdk.client, alice_address).await?;
	println!("RPC Nonce: {}", nonce);

	// Fetch none via Storage
	let alice_account = account::account_id_from_str(alice_address)?;
	let info = account::account_info(&sdk.client, alice_account, None).await?;
	println!("Nonce: {}", info.nonce);

	Ok(())
}
