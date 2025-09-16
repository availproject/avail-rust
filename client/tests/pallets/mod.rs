use avail_rust_client::error::Error;

mod balances;
mod da;
mod multisig;
mod proxy;
mod staking;
mod utility;

pub async fn run_tests() -> Result<(), Error> {
	balances::run_tests().await?;
	da::run_tests().await?;
	multisig::run_tests().await?;
	proxy::run_tests().await?;
	staking::run_tests().await?;
	utility::run_tests().await?;
	Ok(())
}
