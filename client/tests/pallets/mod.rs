use avail_rust_client::error::Error;

mod balances;

pub async fn run_tests() -> Result<(), Error> {
	balances::run_tests().await?;
	Ok(())
}
