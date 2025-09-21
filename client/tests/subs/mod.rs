mod block;
mod block_with_just;
mod client_mock;
mod header;

use avail_rust_client::Error;

pub async fn run_tests() -> Result<(), Error> {
	// block::run_tests().await?;
	// header::run_tests().await?;
	// block_with_just::run_tests().await?;
	Ok(())
}
