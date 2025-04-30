/* use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;

	let alice_account = AccountId::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")?;

	let info = sdk.client.finalized_block_account_info(alice_account).await?;
	println!("Flags: {:?}", info.data.flags);
	println!("Free: {}", info.data.free);
	println!("Frozen: {}", info.data.frozen);
	println!("Reserved: {}", info.data.reserved);

	Ok(())
}
 */
