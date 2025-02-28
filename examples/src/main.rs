mod account;
mod account_balance;
mod account_creation;
mod account_nonce;
mod batch;
mod block;
mod block_data_submission_all;
mod block_data_submission_by_app_id;
mod block_data_submission_by_hash;
mod block_data_submission_by_index;
mod block_data_submission_by_signer;
mod block_events;
mod block_transaction_all;
mod block_transaction_all_static;
mod block_transaction_by_app_id;
mod block_transaction_by_app_id_static;
mod block_transaction_by_hash;
mod block_transaction_by_hash_static;
mod block_transaction_by_index;
mod block_transaction_by_index_static;
mod block_transaction_by_signer;
mod block_transaction_by_signer_static;
mod custom_rpc_connection;
mod data_submission;
mod http_rpc_connection;
mod proxy;
mod rpc;
mod storage;
mod transaction;
mod transaction_execute;
mod transaction_execute_and_watch_finalization;
mod transaction_execute_and_watch_inclusion;
mod transaction_options;
mod transaction_payment;
mod transaction_state;
mod tx_interface;
mod validator;

use avail_rust::{error::ClientError, SDK};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	SDK::enable_logging();

	account::run().await?;
	batch::run().await?;
	block::run().await?;
	data_submission::run().await?;
	proxy::run().await?;
	rpc::run().await?;
	storage::run().await?;
	validator::run().await?;
	http_rpc_connection::run().await?;
	custom_rpc_connection::run().await?;
	transaction_payment::run().await?;
	transaction::run().await?;
	transaction_state::run().await?;

	// TODO
	// tx_interface::run().await?;

	Ok(())
}
