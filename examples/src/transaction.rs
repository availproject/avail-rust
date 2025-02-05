use avail_rust::prelude::*;

use crate::{
	transaction_execute, transaction_execute_and_watch_finalization, transaction_execute_and_watch_inclusion,
	transaction_options, transaction_payment,
};

pub async fn run() -> Result<(), ClientError> {
	transaction_execute_and_watch_finalization::run().await?;
	transaction_execute_and_watch_inclusion::run().await?;
	transaction_execute::run().await?;
	transaction_options::run().await?;
	transaction_payment::run().await?;

	println!("Transaction finished correctly");
	Ok(())
}
