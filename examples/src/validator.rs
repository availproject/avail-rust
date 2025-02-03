/* use avail_rust::{
	prelude::*,
	transactions::staking::{Commission, RewardDestination},
	utils,
};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = SDK::charlie()?;

	// Bond min_validator_bond or 1 AVAIL token
	let storage = sdk.online_client.storage().at_latest().await?;
	let min_validator_bond = storage
		.fetch(&avail::storage().staking().min_validator_bond())
		.await?
		.unwrap_or_else(|| SDK::one_avail());

	let payee = RewardDestination::Staked;

	// Bond
	let tx = sdk.tx.staking.bond(min_validator_bond, payee);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	res.is_successful()
		.ok_or(ClientError::from("Failed to decode events"))?;

	// Generate Session Keys
	let keys = rpc::author::rotate_keys(&sdk.rpc_client).await?;
	let keys = utils::deconstruct_session_keys(keys)?;

	// Set Keys
	let tx = sdk.tx.session.set_keys(keys);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	res.is_successful()
		.ok_or(ClientError::from("Failed to decode events"))?;

	// Validate
	let commission = Commission::new(10)?;
	let tx = sdk.tx.staking.validate(commission, false);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	res.is_successful()
		.ok_or(ClientError::from("Failed to decode events"))?;

	Ok(())
}
 */
