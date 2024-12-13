mod account_nonce;
mod transaction_options;

use avail_rust::error::ClientError;

pub async fn run() -> Result<(), ClientError> {
	account_nonce::run().await?;
	transaction_options::run().await?;

	Ok(())
}
