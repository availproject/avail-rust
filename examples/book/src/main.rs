mod block_api;
mod submission_api;

use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
	block_api::example().await?;
	Ok(())
}
