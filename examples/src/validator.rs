use avail_rust::{
	avail, error::ClientError, rpc, transactions::staking::Commission, utils, RewardDestination,
	SDK,
};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = SDK::alice()?;

	// Bond min_validator_bond or 1 AVAIL token
	let storage = sdk.online_client.storage().at_latest().await?;
	let min_validator_bond = storage
		.fetch(&avail::storage().staking().min_validator_bond())
		.await?
		.unwrap_or_else(|| SDK::one_avail());

	let payee = RewardDestination::Staked;

	// Bond
	let tx = sdk.tx.staking.bond(min_validator_bond, payee);
	_ = tx.execute_and_watch_inclusion(&account, None).await?;

	// Generate Session Keys
	let keys = rpc::author::rotate_keys(&sdk.rpc_client).await?;
	let keys = utils::deconstruct_session_keys(keys)?;

	// Set Keys
	let tx = sdk.tx.session.set_keys(keys);
	_ = tx.execute_and_watch_inclusion(&account, None).await?;

	// Validate
	let commission = Commission::new(10)?;
	let tx = sdk.tx.staking.validate(commission, false);
	_ = tx.execute_and_watch_inclusion(&account, None).await?;

	Ok(())
}
