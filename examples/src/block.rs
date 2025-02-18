use avail_rust::prelude::*;

use crate::{
	block_data_submission_all, block_data_submission_by_app_id, block_data_submission_by_hash,
	block_data_submission_by_index, block_data_submission_by_signer, block_events, block_transaction_all,
	block_transaction_all_static, block_transaction_by_app_id, block_transaction_by_app_id_static,
	block_transaction_by_hash, block_transaction_by_hash_static, block_transaction_by_index,
	block_transaction_by_index_static, block_transaction_by_signer, block_transaction_by_signer_static,
};

pub async fn run() -> Result<(), ClientError> {
	block_data_submission_all::run().await?;
	block_data_submission_by_app_id::run().await?;
	block_data_submission_by_hash::run().await?;
	block_data_submission_by_index::run().await?;
	block_data_submission_by_signer::run().await?;
	block_transaction_all::run().await?;
	block_transaction_by_app_id::run().await?;
	block_transaction_by_hash::run().await?;
	block_transaction_by_index::run().await?;
	block_transaction_by_signer::run().await?;
	block_transaction_all_static::run().await?;
	block_transaction_by_app_id_static::run().await?;
	block_transaction_by_hash_static::run().await?;
	block_transaction_by_index_static::run().await?;
	block_transaction_by_signer_static::run().await?;
	block_events::run().await?;

	println!("Block finished correctly");
	Ok(())
}
