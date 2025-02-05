use super::{Options, Params, TransactionDetails};
use crate::{
	error::ClientError,
	rpc,
	sdk::ClientMode,
	transaction::{logger::Logger, watcher::Watcher},
	Client, WaitFor,
};
use primitive_types::H256;
use subxt::{blocks::StaticExtrinsic, ext::scale_encode::EncodeAsFields, tx::DefaultPayload};
use subxt_signer::sr25519::Keypair;

#[derive(Debug)]
pub enum TransactionExecutionError {
	FailedToSubmitTransaction,
	BlockStreamFailure,
	SubxtError(subxt::Error),
}

impl TransactionExecutionError {
	pub fn to_string(&self) -> String {
		match self {
			TransactionExecutionError::FailedToSubmitTransaction => {
				String::from("Failed to submit transaction").to_string()
			},
			TransactionExecutionError::BlockStreamFailure => String::from("Block Stream Failure").to_string(),
			TransactionExecutionError::SubxtError(error) => error.to_string(),
		}
	}
}

impl From<subxt::Error> for TransactionExecutionError {
	fn from(value: subxt::Error) -> Self {
		Self::SubxtError(value)
	}
}

/// Creates and signs an extrinsic and submits to the chain for block inclusion.
///
/// Returns `Ok` with the extrinsic hash if it is valid extrinsic.
///
/// # Note
///
/// Success does not mean the extrinsic has been included in the block, just that it is valid
/// and has been included in the transaction pool.
pub async fn sign_and_send<T>(
	client: &Client,
	account: &Keypair,
	call: &DefaultPayload<T>,
	options: Options,
) -> Result<H256, ClientError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = account.public_key().to_account_id();
	let options = options.build(&client, &account_id).await?;
	let params = options.build().await?;

	sign_and_send_raw_params(client, account, call, params).await
}

pub async fn sign_and_send_raw_params<T>(
	client: &Client,
	signer: &Keypair,
	call: &DefaultPayload<T>,
	params: Params,
) -> Result<H256, ClientError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	if params.6 .0 .0 != 0 && (call.pallet_name() != "DataAvailability" || call.call_name() != "submit_data") {
		return Err(ClientError::from("Transaction is not compatible with non-zero AppIds"));
	}

	let logger = Logger::new(H256::default(), true);
	logger.log_tx_submitting(signer, call, &params, client.mode);

	match client.mode {
		ClientMode::HTTP => {
			let tx_client = client.online_client.tx();
			let signed_call = tx_client.create_signed(call, signer, params).await?;
			let extrinsic = signed_call.encoded();
			let tx_hash = rpc::author::submit_extrinsic(client, extrinsic).await?;
			Ok(tx_hash)
		},
		ClientMode::WS => {
			let tx_hash = client.online_client.tx().sign_and_submit(call, signer, params).await?;
			Ok(tx_hash)
		},
	}
}

pub async fn sign_send_and_watch<T>(
	client: &Client,
	account: &Keypair,
	call: &DefaultPayload<T>,
	wait_for: WaitFor,
	options: Options,
) -> Result<TransactionDetails, ClientError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = account.public_key().to_account_id();
	let mut options = options.build(&client, &account_id).await?;
	let mut retry_count = 2;

	loop {
		let params = options.build().await?;
		let tx_hash = sign_and_send_raw_params(client, account, call, params).await?;

		let logger = Logger::new(tx_hash, true);
		logger.log_tx_submitted(&account, &options.mortality);

		let watcher = Watcher::new(client.clone(), tx_hash);
		let watcher = watcher.logger(logger.clone()).wait_for(wait_for);

		let tx_details = watcher.run().await?;
		if let Some(tx_details) = tx_details {
			return Ok(tx_details);
		}

		if retry_count == 0 {
			logger.log_tx_retry_abort();
			return Err(ClientError::TransactionExecution(
				TransactionExecutionError::FailedToSubmitTransaction,
			));
		}

		options.regenerate_mortality(client).await?;

		retry_count -= 1;
		logger.log_tx_retry();
	}
}
