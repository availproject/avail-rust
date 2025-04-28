use super::{Options, Params, PopulatedOptions, TransactionDetails};
use crate::{block::EventRecords, rpc, transaction::logger::Logger, ABlock, AccountId, Client, WaitFor};
use primitive_types::H256;
use std::{sync::Arc, time::Duration};
use subxt::{blocks::StaticExtrinsic, ext::scale_encode::EncodeAsFields, tx::DefaultPayload};
use subxt_signer::sr25519::Keypair;

#[derive(Debug)]
pub enum SubmissionStateError {
	FailedToSubmit { reason: subxt::Error },
	SubmittedButErrorInSearch { tx_hash: H256, reason: subxt::Error },
	Dropped { tx_hash: H256 },
}

impl SubmissionStateError {
	pub fn to_string(&self) -> String {
		match self {
			Self::FailedToSubmit { reason } => {
				std::format!("Failed to Submit Transaction. Reason: {}", reason.to_string())
			},
			Self::SubmittedButErrorInSearch { reason, tx_hash } => {
				std::format!(
					"Submitted transaction but error occurred while searching for it. Tx Hash: {:?},  Reason: {}",
					tx_hash,
					reason.to_string()
				)
			},
			Self::Dropped { tx_hash } => {
				std::format!("Submitted transaction has been dropped. Tx Hash: {:?}", tx_hash)
			},
		}
	}
}

#[derive(Debug, Clone)]
pub struct SubmittedTransaction {
	pub hash: H256,
	pub account_id: AccountId,
	pub options: PopulatedOptions,
}

/// TODO
pub async fn sign_and_submit<T>(
	client: &Client,
	account: &Keypair,
	call: &DefaultPayload<T>,
	options: Options,
) -> Result<SubmittedTransaction, subxt::Error>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = account.public_key().to_account_id();
	let options = options.build(client, &account_id).await?;
	let params = options.clone().build().await;

	let hash = sign_and_submit_raw_params(client, account, call, params).await?;
	Ok(SubmittedTransaction {
		hash,
		account_id,
		options,
	})
}

/// TODO
pub async fn sign_and_submit_raw_params<T>(
	client: &Client,
	signer: &Keypair,
	call: &DefaultPayload<T>,
	params: Params,
) -> Result<H256, subxt::Error>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	if params.6 .0 .0 != 0 && (call.pallet_name() != "DataAvailability" || call.call_name() != "submit_data") {
		return Err(subxt::Error::Other(
			"Transaction is not compatible with non-zero AppIds".into(),
		));
	}

	/* 	let logger = Logger::new(H256::default(), true);
	logger.log_tx_submitting(signer, call, &params, options.mode); */

	let tx_client = client.online_client.tx();
	let signed_call = tx_client.create_signed(call, signer, params).await?;
	let extrinsic = signed_call.encoded();
	let tx_hash = rpc::author::submit_extrinsic(client, extrinsic).await?;
	Ok(tx_hash)
}

/// TODO
pub async fn sign_submit_and_watch<T>(
	client: &Client,
	account: &Keypair,
	call: &DefaultPayload<T>,
	wait_for: WaitFor,
	options: Options,
) -> Result<TransactionDetails, SubmissionStateError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = account.public_key().to_account_id();
	let info = match sign_and_submit(client, account, call, options).await {
		Ok(x) => x,
		Err(err) => return Err(SubmissionStateError::FailedToSubmit { reason: err }),
	};

	let account = (account_id.clone(), info.options.nonce as u32);
	let mortality = (
		info.options.mortality.period as u32,
		info.options.mortality.block_number,
	);

	let sleep_duration = Duration::from_secs(3);
	let block_id = match wait_for {
		WaitFor::BlockInclusion => find_block_id_best_block(client, account, mortality, sleep_duration).await,
		WaitFor::BlockFinalization => find_block_id_finalized(client, account, mortality, sleep_duration).await,
	};

	let block_id = match block_id {
		Ok(x) => x,
		Err(err) => {
			return Err(SubmissionStateError::SubmittedButErrorInSearch {
				tx_hash: info.hash,
				reason: err,
			})
		},
	};

	let Some(block_id) = block_id else {
		return Err(SubmissionStateError::Dropped { tx_hash: info.hash });
	};

	let mut block = None;
	if let Ok(cache) = client.cache.lock() {
		if let Some(cached_block) = &cache.last_fetched_block {
			if cached_block.0 == block_id.0 {
				block = Some(cached_block.1.clone())
			}
		}
	}

	let block: Arc<ABlock> = if let Some(block) = block {
		block
	} else {
		let block = match client.block_at(block_id.0).await {
			Ok(x) => x,
			Err(err) => {
				return Err(SubmissionStateError::SubmittedButErrorInSearch {
					tx_hash: info.hash,
					reason: err,
				})
			},
		};
		let block = Arc::new(block);
		if let Ok(mut cache) = client.cache.lock() {
			cache.last_fetched_block = Some((block_id.0, block.clone()))
		}
		block
	};

	let details = match find_transaction(client, &block, &info.hash).await {
		Ok(x) => x,
		Err(err) => {
			return Err(SubmissionStateError::SubmittedButErrorInSearch {
				tx_hash: info.hash,
				reason: err,
			})
		},
	};

	match details {
		Some(x) => Ok(x),
		None => Err(SubmissionStateError::Dropped { tx_hash: info.hash }),
	}
}

/// TODO
pub async fn find_transaction(
	client: &Client,
	block: &ABlock,
	tx_hash: &H256,
) -> Result<Option<TransactionDetails>, subxt::Error> {
	let transactions = block.extrinsics().await?;
	let tx_found = transactions.iter().find(|e| e.hash() == *tx_hash);
	let Some(ext_details) = tx_found else {
		return Ok(None);
	};

	let events = match ext_details.events().await.ok() {
		Some(x) => EventRecords::new_ext(x),
		None => None,
	};

	let value = TransactionDetails::new(
		client.clone(),
		events,
		*tx_hash,
		ext_details.index(),
		block.hash(),
		block.number(),
	);

	Ok(Some(value))
}

/// TODO
pub async fn find_block_id_finalized(
	client: &Client,
	account: (AccountId, u32),
	mortality: (u32, u32),
	sleep_duration: Duration,
) -> Result<Option<(H256, u32)>, subxt::Error> {
	let (period, fork_height) = mortality;
	let mortality_ends_height = fork_height + period;
	let address = std::format!("{}", account.0);

	let mut next_block_height = fork_height + 1;
	let mut block_height = client.finalized_block_number().await?;

	while mortality_ends_height >= next_block_height {
		if next_block_height > block_height {
			tokio::time::sleep(sleep_duration).await;
			block_height = client.finalized_block_number().await?;
			continue;
		}

		let next_block_hash = client.block_hash(next_block_height).await?;
		let state_nonce = crate::account::nonce_state(client, &address, Some(next_block_hash)).await?;
		if state_nonce > account.1 {
			return Ok(Some((next_block_hash, next_block_height)));
		}

		next_block_height += 1;
	}

	Ok(None)
}

/// TODO
pub async fn find_block_id_best_block(
	client: &Client,
	account: (AccountId, u32),
	mortality: (u32, u32),
	sleep_duration: Duration,
) -> Result<Option<(H256, u32)>, subxt::Error> {
	let (period, fork_height) = mortality;
	let mortality_ends_height = fork_height + period;
	let address = std::format!("{}", account.0);

	let mut next_block_height = fork_height + 1;
	let mut block_height = client.best_block_number().await?;

	while mortality_ends_height >= next_block_height {
		if next_block_height > block_height {
			tokio::time::sleep(sleep_duration).await;
			block_height = client.best_block_number().await?;
			continue;
		}

		let next_block_hash = client.block_hash(next_block_height).await?;
		let state_nonce = crate::account::nonce_state(client, &address, Some(next_block_hash)).await?;
		if state_nonce > account.1 {
			return Ok(Some((next_block_hash, next_block_height)));
		}

		next_block_height += 1;
	}

	Ok(None)
}
