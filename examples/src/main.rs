mod account;
mod batch;
mod block_indexing;
mod parallel_transaction_submissions;
mod storage;
mod transaction_submission;

// mod subxt_metadata;

use avail_rust::prelude::{Client, ClientError};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_logging();
	account::run().await?;
	transaction_submission::run().await?;
	parallel_transaction_submissions::run().await?;
	batch::run().await?;
	block_indexing::run().await?;
	storage::run().await?;

	// subxt_metadata::run().await?;

	Ok(())
}
