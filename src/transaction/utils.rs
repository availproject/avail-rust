use super::{Options, Params, TransactionDetails};
use crate::{block::EventRecords, error::ClientError, rpc, Client, WaitFor};
use log::{info, log_enabled, warn};
use primitive_types::H256;
use std::time::Duration;
use subxt::{blocks::StaticExtrinsic, ext::scale_encode::EncodeAsFields, tx::DefaultPayload};
use subxt_signer::sr25519::Keypair;

#[derive(Debug)]
pub enum TransactionExecutionError {
	TransactionNotFound,
	BlockStreamFailure,
	SubxtError(subxt::Error),
}

impl TransactionExecutionError {
	pub fn to_string(&self) -> String {
		match self {
			TransactionExecutionError::TransactionNotFound => String::from("Transaction not found").to_string(),
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
	options: Option<Options>,
) -> Result<H256, ClientError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = account.public_key().to_account_id();
	let options = options.unwrap_or_default().build(&client, &account_id).await?;

	let params = options.build().await?;

	sign_and_send_raw_params(client, account, call, params).await
}

pub async fn sign_and_send_raw_params<T>(
	client: &Client,
	account: &Keypair,
	call: &DefaultPayload<T>,
	params: Params,
) -> Result<H256, ClientError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	if params.6 .0 .0 != 0 && call.pallet_name() != "DataAvailability" && call.call_name() != "submit_data" {
		return Err(ClientError::from("Transaction is not compatible with non-zero AppIds"));
	}

	if log_enabled!(log::Level::Info) {
		let address = account.public_key().to_account_id().to_string();
		let call_name = call.call_name();
		let pallet_name = call.pallet_name();
		let nonce = &params.4 .0;
		let app_id = &params.6 .0;
		info!(
			target: "transaction",
			"Signing and submitting new transaction. Account: {}, Nonce: {:?}, Pallet Name: {}, Call Name: {}, App Id: {}",
			address, nonce, pallet_name, call_name, app_id
		);
	}

	let tx_hash = client.online_client.tx().sign_and_submit(call, account, params).await?;

	Ok(tx_hash)
}

pub async fn watch(
	client: &Client,
	tx_hash: H256,
	wait_for: WaitFor,
	block_timeout: Option<u32>,
) -> Result<TransactionDetails, TransactionExecutionError> {
	let mut block_hash;
	let mut block_number;
	let tx_details;

	let mut stream = match wait_for == WaitFor::BlockInclusion {
		true => client.blocks().subscribe_all().await,
		false => client.blocks().subscribe_finalized().await,
	}?;

	let mut current_block_number: Option<u32> = None;
	let mut timeout_block_number: Option<u32> = None;

	if log_enabled!(log::Level::Info) {
		let marker = &format!("{:?}", tx_hash)[0..10];
		info!(target: "watcher", "{}: Watching for Tx Hash: {:?}. Waiting for: {}, Block timeout: {:?}", marker, tx_hash, wait_for.to_str(), block_timeout);
	}
	loop {
		let Some(block) = stream.next().await else {
			return Err(TransactionExecutionError::BlockStreamFailure);
		};

		let block = match block {
			Ok(b) => b,
			Err(e) => {
				if e.is_disconnected_will_reconnect() {
					warn!("The RPC connection was lost and we may have missed a few blocks");
					continue;
				}

				return Err(TransactionExecutionError::SubxtError(e));
			},
		};
		block_hash = block.hash();
		block_number = block.number();

		if log_enabled!(log::Level::Info) {
			let marker = &format!("{:?}", tx_hash)[0..10];
			info!(target: "watcher", "{}: New block fetched. Hash: {:?}, Number: {}", marker, block_hash, block_number);
		}

		let transactions = block.extrinsics().await?;
		let tx_found = transactions.iter().find(|e| e.hash() == tx_hash);
		if let Some(tx) = tx_found {
			tx_details = tx;
			break;
		}

		// Block timeout logic
		let Some(block_timeout) = block_timeout else {
			continue;
		};

		if current_block_number.is_none() {
			current_block_number = Some(block_number);
			timeout_block_number = Some(block_number + block_timeout);

			if log_enabled!(log::Level::Info) {
				let marker = &format!("{:?}", tx_hash)[0..10];
				info!(target: "watcher", "{}: Current Block Number: {}, Timeout Block Number: {}", marker, block_number, block_number + block_timeout + 1);
			}
		}
		if timeout_block_number.is_some_and(|timeout| block_number > timeout) {
			return Err(TransactionExecutionError::TransactionNotFound);
		}
	}

	let events = match tx_details.events().await.ok() {
		Some(x) => EventRecords::new_ext(x),
		None => None,
	};
	let tx_index = tx_details.index();

	if log_enabled!(log::Level::Info) {
		let marker = &format!("{:?}", tx_hash)[0..10];
		info!(target: "watcher", "{}: Transaction was found. Tx Hash: {:?}, Tx Index: {}, Block Hash: {:?}, Block Number: {}", marker, tx_hash, tx_index, block_hash, block_number);
	}

	Ok(TransactionDetails::new(
		client.clone(),
		events,
		tx_hash,
		tx_index,
		block_hash,
		block_number,
	))
}

pub async fn sign_send_and_watch<T>(
	client: &Client,
	account: &Keypair,
	call: &DefaultPayload<T>,
	wait_for: WaitFor,
	options: Option<Options>,
	block_timeout: Option<u32>,
	retry_count: Option<u32>,
) -> Result<TransactionDetails, ClientError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = account.public_key().to_account_id();

	let options = options.unwrap_or_default();
	let mut options = options.build(&client, &account_id).await?;

	let mut retry_count = retry_count.unwrap_or(0);
	let retry_count_max = retry_count;
	loop {
		let params = options.build().await?;
		let tx_hash = sign_and_send_raw_params(client, account, call, params).await?;
		if log_enabled!(log::Level::Info) {
			let address = account.public_key().to_account_id().to_string();
			let mortality = options.mortality.block_number + options.mortality.period as u32;
			let marker = &format!("{:?}", tx_hash)[0..10];
			info!(
				target: "transaction",
				"{}: Transaction was submitted. Account: {}, TxHash: {:?}, Mortality Block: {:?}",
				marker,
				address,
				tx_hash,
				mortality
			);
		}

		let result = watch(client, tx_hash, wait_for, block_timeout).await;
		let error = match result {
			Ok(details) => return Ok(details),
			Err(err) => err,
		};

		match error {
			TransactionExecutionError::TransactionNotFound => (),
			TransactionExecutionError::BlockStreamFailure => return Err(ClientError::TransactionExecution(error)),
			TransactionExecutionError::SubxtError(_) => return Err(ClientError::TransactionExecution(error)),
		};

		if retry_count == 0 {
			if log_enabled!(log::Level::Warn) {
				let marker = &format!("{:?}", tx_hash)[0..10];
				warn!(target: "watcher", "{}: Failed to find transaction. Tx Hash: {:?}. Aborting", marker, tx_hash);
			}
			return Err(ClientError::TransactionExecution(error));
		}

		options.regenerate_mortality(client).await?;

		retry_count -= 1;
		info!(target: "watcher", "Failed to find transaction. Tx Hash: {:?}. Trying again. {:?}/{:?}", tx_hash, retry_count_max, retry_count);
	}
}

pub async fn http_sign_and_send<T>(
	client: &Client,
	account: &Keypair,
	call: &DefaultPayload<T>,
	options: Option<Options>,
) -> Result<H256, ClientError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = account.public_key().to_account_id();
	let options = options.unwrap_or_default().build(client, &account_id).await?;

	let params = options.build().await?;

	http_sign_and_send_raw_params(client, account, call, params).await
}

pub async fn http_sign_and_send_raw_params<T>(
	client: &Client,
	account: &Keypair,
	call: &DefaultPayload<T>,
	params: Params,
) -> Result<H256, ClientError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	if params.6 .0 .0 != 0 && call.pallet_name() != "DataAvailability" && call.call_name() != "submit_data" {
		return Err(ClientError::from("Transaction is not compatible with non-zero AppIds"));
	}

	if log_enabled!(log::Level::Debug) {
		let address = account.public_key().to_account_id().to_string();
		let call_name = call.call_name();
		let pallet_name = call.pallet_name();
		let nonce = &params.4 .0;
		let app_id = &params.6 .0;
		info!(
			target: "transaction",
			"Signing and submitting new transaction. Account: {}, Nonce: {:?}, Pallet Name: {}, Call Name: {}, App Id: {}",
			address, nonce, pallet_name, call_name, app_id
		);
	}

	let tx_client = client.online_client.tx();
	let signed_call = tx_client.create_signed(call, account, params).await?;
	let extrinsic = signed_call.encoded();
	let tx_hash = rpc::author::submit_extrinsic(client, extrinsic).await?;

	Ok(tx_hash)
}

pub async fn http_watch(
	client: &Client,
	tx_hash: H256,
	wait_for: WaitFor,
	sleep_duration: Duration,
	block_timeout: Option<u32>,
) -> Result<TransactionDetails, TransactionExecutionError> {
	let mut current_block_hash: Option<H256> = None;
	let mut timeout_block_number: Option<u32> = None;
	let mut block_hash;
	let mut block_number;
	let tx_details;
	let mut should_sleep = false;

	info!(target: "watcher", "Watching for Tx Hash: {:?}. Waiting for: {}, Block timeout: {:?}", tx_hash, wait_for.to_str(), block_timeout);

	loop {
		if should_sleep {
			tokio::time::sleep(sleep_duration).await;
		}
		if !should_sleep {
			should_sleep = true;
		}

		block_hash = match wait_for {
			WaitFor::BlockInclusion => rpc::chain::get_block_hash(client, None).await.unwrap(),
			WaitFor::BlockFinalization => rpc::chain::get_finalized_head(client).await.unwrap(),
		};

		if current_block_hash.is_some_and(|x| x == block_hash) {
			continue;
		}
		current_block_hash = Some(block_hash);

		let blocks = client.blocks();
		let block = blocks.at(block_hash).await?;
		block_number = block.number();
		block_hash = block.hash();
		info!(target: "watcher", "New block fetched. Hash: {:?}, Number: {}", block_hash, block_number);

		let transactions = block.extrinsics().await?;
		let tx_found = transactions.iter().find(|e| e.hash() == tx_hash);
		if let Some(tx) = tx_found {
			tx_details = tx;
			break;
		}

		// Block timeout logic
		let Some(block_timeout) = block_timeout else {
			continue;
		};

		if timeout_block_number.is_none() {
			timeout_block_number = Some(block_number + block_timeout);
			info!(target: "watcher", "Current Block Number: {}, Timeout Block Number: {}", block_number, block_number + block_timeout + 1);
		}
		if timeout_block_number.is_some_and(|timeout| block_number > timeout) {
			return Err(TransactionExecutionError::TransactionNotFound);
		}
	}

	let events = match tx_details.events().await.ok() {
		Some(x) => EventRecords::new_ext(x),
		None => None,
	};
	let tx_index = tx_details.index();

	info!(target: "watcher", "Transaction was found. Tx Hash: {:?}, Tx Index: {}, Block Hash: {:?}, Block Number: {}", tx_hash, tx_index, block_hash, block_number);

	Ok(TransactionDetails::new(
		client.clone(),
		events,
		tx_hash,
		tx_index,
		block_hash,
		block_number,
	))
}

pub async fn http_sign_send_and_watch<T>(
	client: &Client,
	account: &Keypair,
	call: &DefaultPayload<T>,
	wait_for: WaitFor,
	options: Option<Options>,
	block_timeout: Option<u32>,
	retry_count: Option<u32>,
	sleep_duration: Option<Duration>,
) -> Result<TransactionDetails, ClientError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = account.public_key().to_account_id();

	let options = options.unwrap_or_default();
	let mut options = options.build(client, &account_id).await?;

	let mut retry_count = retry_count.unwrap_or(0);
	let sleep_duration = sleep_duration.unwrap_or_else(|| Duration::from_secs(3));
	loop {
		let params = options.build().await?;
		let tx_hash = http_sign_and_send_raw_params(client, account, call, params).await?;
		let result = http_watch(client, tx_hash, wait_for, sleep_duration, block_timeout).await;
		let error = match result {
			Ok(details) => return Ok(details),
			Err(err) => err,
		};

		match error {
			TransactionExecutionError::TransactionNotFound => (),
			TransactionExecutionError::BlockStreamFailure => return Err(ClientError::TransactionExecution(error)),
			TransactionExecutionError::SubxtError(_) => return Err(ClientError::TransactionExecution(error)),
		};

		if retry_count == 0 {
			warn!(target: "watcher", "Failed to find transaction. Tx Hash: {:?}. Aborting", tx_hash);
			return Err(ClientError::TransactionExecution(error));
		}

		options.regenerate_mortality(client).await?;

		info!(target: "watcher", "Failed to find transaction. Tx Hash: {:?}. Trying again.", tx_hash);
		retry_count -= 1;
	}
}
