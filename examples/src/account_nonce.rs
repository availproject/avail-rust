use avail_rust::{account, error::ClientError, SDK};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let alice_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

	// Fetch nonce via RPC
	let nonce = account::fetch_nonce(&sdk.rpc_client, alice_address).await?;
	println!("RPC Nonce: {}", nonce);

	// Fetch nonce via state
	let nonce =
		account::fetch_nonce_state(&sdk.online_client, &sdk.rpc_client, alice_address, None)
			.await?;
	println!("State Nonce: {}", nonce);

	Ok(())
}

/*
	Example Output:

	Nonce from Node: 1
	Nonce from best block state: 1
	Nonce from custom block state: 1
	Nonce from  manually reading storage: 1
*/
