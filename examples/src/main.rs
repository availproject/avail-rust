mod account;
mod account_balance;
mod account_creation;
mod account_nonce;
mod batch;
mod block;
mod custom_rpc_connection;
mod data_submission;
mod events;
mod http_rpc_connection;
mod rpc;
mod storage;
mod transaction_options;
mod transaction_payment;
mod transactions;
mod tx_interface;
mod validator;

use avail_rust::{error::ClientError, SDK};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	SDK::enable_logging();

	account::run().await?;
	transaction_payment::run().await?;
	block::run().await?;
	batch::run().await?;
	custom_rpc_connection::run().await?;
	data_submission::run().await?;
	events::run().await?;
	http_rpc_connection::run().await?;
	rpc::run().await?;
	storage::run().await?;
	transaction_options::run().await?;
	transactions::run().await?;
	tx_interface::run().await?;
	validator::run().await?;

	Ok(())
}

/* fn assert_true(v1: bool, msg: &str) {
	if v1 == false {
		panic!("{}", msg);
	}
}

fn assert_eq<T: Eq>(v1: T, v2: T, msg: &str) {
	if v1.ne(&v2) {
		panic!("{}", msg);
	}
}
 */
