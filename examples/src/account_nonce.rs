/* use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;

	let alice_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

	// Fetch nonce via RPC
	let nonce = sdk.client.nonce(alice_address).await?;
	println!("RPC Nonce: {}", nonce);

	// Fetch none via Storage
	let alice_account = AccountId::from_str(alice_address)?;
	let info = sdk.client.finalized_block_account_info(alice_account).await?;
	println!("Nonce: {}", info.nonce);

	Ok(())
}
 */
