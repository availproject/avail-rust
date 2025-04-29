use avail_rust::{
	prelude::*,
	transactions::staking::{Commission, RewardDestination},
};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = account::charlie();

	// Bond min_validator_bond or 1 AVAIL token
	let storage = sdk.client.subxt_storage().at_latest().await?;
	let min_validator_bond = storage
		.fetch(&avail::storage().staking().min_validator_bond())
		.await?
		.unwrap_or_else(|| SDK::one_avail());

	let payee = RewardDestination::Staked;

	// Bond
	let tx = sdk.tx.staking.bond(min_validator_bond, payee);
	let res = tx.execute_and_watch(&account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Generate Session Keys
	let keys = sdk.client.rpc_author_rotate_keys().await?;

	// Set Keys
	let tx = sdk.tx.session.set_keys(keys);
	let res = tx.execute_and_watch(&account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Validate
	let commission = Commission::new(10)?;
	let tx = sdk.tx.staking.validate(commission, false);
	let res = tx.execute_and_watch(&account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));

	Ok(())
}
