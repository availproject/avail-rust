mod block;
mod block_with_just;
mod extrinsic;
mod grandpa_justification;
mod grandpa_justification_json;
mod header;
mod raw_extrinsic;
mod transaction;

use avail_rust_client::Error;

pub async fn run_tests() -> Result<(), Error> {
	block::run_tests().await?;
	header::run_tests().await?;
	block_with_just::run_tests().await?;
	grandpa_justification::run_tests().await?;
	grandpa_justification_json::run_tests().await?;
	transaction::run_tests().await?;
	extrinsic::run_tests().await?;
	raw_extrinsic::run_tests().await?;
	Ok(())
}
