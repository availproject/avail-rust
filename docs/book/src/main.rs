mod basics;
mod examples;
mod rpc;
mod storage;
mod test;

use avail_rust::{error::ClientError, SDK};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	SDK::enable_logging();

	// storage::run().await?;
	// rpc::run().await?;
	// basics::run().await?;
	// examples::run().await?;
	test::run().await?;

	Ok(())
}
