mod account;
mod batch;
mod parallel_transaction_submissions;
mod transaction_submission;
// mod batch;
// mod block;
// mod block_data_submission_all;
// mod block_data_submission_by_app_id;
// mod block_data_submission_by_hash;
// mod block_data_submission_by_index;
// mod block_data_submission_by_signer;
// mod block_events;
// mod block_transaction_all;
// mod block_transaction_all_static;
// mod block_transaction_by_app_id;
// mod block_transaction_by_app_id_static;
// mod block_transaction_by_hash;
// mod block_transaction_by_hash_static;
// mod block_transaction_by_index;
// mod block_transaction_by_index_static;
// mod block_transaction_by_signer;
// mod block_transaction_by_signer_static;
// mod data_submission;
// mod http_rpc_connection;
// mod indexer;
// mod proxy;
// mod rpc;
// mod storage;
// mod transaction;
// mod transaction_execute;
// mod transaction_options;
// mod transaction_payment;
// mod transaction_state;
// mod turbo_da;
// mod validator;

use avail_rust::prelude::{Client, ClientError};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_logging();
	// account::run().await?;
	transaction_submission::run().await?;
	// parallel_transaction_submissions::run().await?;
	// batch::run().await?;

	Ok(())
}
