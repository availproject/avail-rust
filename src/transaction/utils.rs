use super::{Options, Params, PopulatedOptions, TransactionDetails};
use crate::{block::EventRecords, rpc, ABlock, AccountId, Client};
use log::info;
use primitive_types::H256;
use std::{sync::Arc, time::Duration};
use subxt::{blocks::StaticExtrinsic, ext::scale_encode::EncodeAsFields, tx::DefaultPayload};
use subxt_signer::sr25519::Keypair;

#[derive(Clone, Copy)]
pub struct BlockId {
	pub hash: H256,
	pub height: u32,
}

impl From<(H256, u32)> for BlockId {
	fn from(value: (H256, u32)) -> Self {
		Self {
			hash: value.0,
			height: value.1,
		}
	}
}

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
				std::format!("Failed to Submit Transaction. Reason: {}", reason)
			},
			Self::SubmittedButErrorInSearch { reason, tx_hash } => {
				std::format!(
					"Submitted transaction but error occurred while searching for it. Tx Hash: {:?},  Reason: {}",
					tx_hash,
					reason
				)
			},
			Self::Dropped { tx_hash } => {
				std::format!("Submitted transaction has been dropped. Tx Hash: {:?}", tx_hash)
			},
		}
	}
}

#[derive(Clone)]
pub struct SubmittedTransaction {
	client: Client,
	pub tx_hash: H256,
	pub account_id: AccountId,
	pub options: PopulatedOptions,
}

impl SubmittedTransaction {
	pub fn nonce(&self) -> u32 {
		self.options.nonce as u32
	}

	pub fn fork_hash(&self) -> H256 {
		self.options.mortality.block_hash
	}

	pub fn fork_height(&self) -> u32 {
		self.options.mortality.block_number
	}

	pub fn mortality_period(&self) -> u32 {
		self.options.mortality.period as u32
	}

	pub async fn find_block_id(&self, sleep_duration: Duration) -> Result<Option<BlockId>, subxt::Error> {
		find_block_id(
			&self.client,
			&self.account_id,
			self.nonce(),
			self.mortality_period(),
			self.fork_height(),
			sleep_duration,
		)
		.await
	}

	/* 	pub async fn find_block_id_best_block(&self, sleep_duration: Duration) -> Result<Option<BlockId>, subxt::Error> {
		find_block_id_best_block(
			&self.client,
			&self.account_id,
			self.nonce(),
			self.mortality_period(),
			self.fork_height(),
			sleep_duration,
		)
		.await
	} */
}

/// TODO
pub async fn sign_and_submit<T>(
	client: &Client,
	signer: &Keypair,
	payload: &DefaultPayload<T>,
	options: Options,
) -> Result<SubmittedTransaction, subxt::Error>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = signer.public_key().to_account_id();
	let options = options.build(client, &account_id).await?;
	let params = options.clone().build().await;
	let tx_hash = sign_and_submit_raw_params(client, signer, payload, params).await?;

	info!(target: "submission", "Transaction submitted. Tx Hash: {:?}, Fork Hash: {:?}, Fork Height: {:?}, Period: {}, Nonce: {}, Account Address: {}", tx_hash, options.mortality.block_hash, options.mortality.block_number, options.mortality.period, options.nonce, account_id);

	Ok(SubmittedTransaction {
		client: client.clone(),
		tx_hash,
		account_id,
		options,
	})
}

/// TODO
pub async fn sign_and_submit_raw_params<T>(
	client: &Client,
	signer: &Keypair,
	payload: &DefaultPayload<T>,
	params: Params,
) -> Result<H256, subxt::Error>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	if params.6 .0 .0 != 0 && (payload.pallet_name() != "DataAvailability" || payload.call_name() != "submit_data") {
		return Err(subxt::Error::Other(
			"Transaction is not compatible with non-zero AppIds".into(),
		));
	}

	let tx_client = client.online_client.tx();
	let signed_call = tx_client.create_signed(payload, signer, params).await?;
	rpc::author::submit_extrinsic(client, signed_call.encoded()).await
}

/// TODO
pub async fn sign_submit_and_watch<T>(
	client: &Client,
	signer: &Keypair,
	payload: &DefaultPayload<T>,
	options: Options,
) -> Result<TransactionDetails, SubmissionStateError>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = signer.public_key().to_account_id();
	let info = match sign_and_submit(client, signer, payload, options).await {
		Ok(x) => x,
		Err(err) => return Err(SubmissionStateError::FailedToSubmit { reason: err }),
	};

	let sleep_duration = Duration::from_secs(3);
	let block_id = info.find_block_id(sleep_duration).await;

	let block_id = match block_id {
		Ok(x) => x,
		Err(err) => {
			return Err(SubmissionStateError::SubmittedButErrorInSearch {
				tx_hash: info.tx_hash,
				reason: err,
			})
		},
	};

	let Some(block_id) = block_id else {
		return Err(SubmissionStateError::Dropped { tx_hash: info.tx_hash });
	};

	let mut block = None;
	if let Ok(cache) = client.cache.lock() {
		if let Some(cached_block) = &cache.last_fetched_block {
			if cached_block.0 == block_id.hash {
				block = Some(cached_block.1.clone())
			}
		}
	}

	let block: Arc<ABlock> = if let Some(block) = block {
		block
	} else {
		let block = match client.block_at(block_id.hash).await {
			Ok(x) => x,
			Err(err) => {
				return Err(SubmissionStateError::SubmittedButErrorInSearch {
					tx_hash: info.tx_hash,
					reason: err,
				})
			},
		};
		let block = Arc::new(block);
		if let Ok(mut cache) = client.cache.lock() {
			cache.last_fetched_block = Some((block_id.hash, block.clone()))
		}
		block
	};

	let details = match find_transaction(client, &block, &info.tx_hash).await {
		Ok(x) => x,
		Err(err) => {
			return Err(SubmissionStateError::SubmittedButErrorInSearch {
				tx_hash: info.tx_hash,
				reason: err,
			})
		},
	};

	match details {
		Some(x) => {
			info!(target: "tx_search", "Transaction Found. Tx Hash: {:?}, Tx Index: {}, Block Hash: {:?}, Block Height: {}, Nonce: {}, Account Address: {}", x.tx_hash, x.tx_index, x.block_hash, x.block_number, info.nonce(), account_id);
			Ok(x)
		},
		None => Err(SubmissionStateError::Dropped { tx_hash: info.tx_hash }),
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
pub async fn find_block_id(
	client: &Client,
	account_id: &AccountId,
	nonce: u32,
	mortality_period: u32,
	fork_height: u32,
	sleep_duration: Duration,
) -> Result<Option<BlockId>, subxt::Error> {
	let mortality_ends_height = fork_height + mortality_period;
	let address = std::format!("{}", account_id);

	let mut next_block_height = fork_height + 1;
	let mut block_height = client.finalized_block_number().await?;

	info!(target: "nonce_search", "Nonce: {} Account address: {} Current Finalized Height: {} Mortality End Height: {}", nonce, account_id, block_height, mortality_ends_height);
	while mortality_ends_height >= next_block_height {
		if next_block_height > block_height {
			tokio::time::sleep(sleep_duration).await;
			block_height = client.finalized_block_number().await?;
			continue;
		}

		let next_block_hash = client.block_hash(next_block_height).await?;
		let state_nonce = crate::account::nonce_state(client, &address, Some(next_block_hash)).await?;
		if state_nonce > nonce {
			info!(target: "nonce_search", "At block height {} and hash {:?} found nonce: {} which is greater than {} for Account address {}. Search is done", next_block_height, next_block_hash, state_nonce, nonce, account_id);
			return Ok(Some(BlockId::from((next_block_hash, next_block_height))));
		}

		info!(target: "nonce_search", "Looking for nonce > than: {} for Account address {}. At block height {} and hash {:?} found nonce: {}", nonce, account_id, next_block_height, next_block_hash, state_nonce);
		next_block_height += 1;
	}

	Ok(None)
}

/* /// TODO
pub async fn find_block_id_best_block(
	client: &Client,
	account_id: &AccountId,
	nonce: u32,
	mortality_period: u32,
	fork_height: u32,
	sleep_duration: Duration,
) -> Result<Option<BlockId>, subxt::Error> {
	let mortality_ends_height = fork_height + mortality_period;
	let address = std::format!("{}", account_id);

	let mut next_block_height = fork_height + 1;
	let mut block_height = client.best_block_number().await?;

	info!(target: "nonce_search", "Nonce: {} Account address: {} Current Finalized Height: {} Mortality End Height: {}", nonce, account_id, block_height, mortality_ends_height);
	while mortality_ends_height >= next_block_height {
		if next_block_height > block_height {
			tokio::time::sleep(sleep_duration).await;
			block_height = client.best_block_number().await?;
			continue;
		}

		let next_block_hash = client.block_hash(next_block_height).await?;
		let state_nonce = crate::account::nonce_state(client, &address, Some(next_block_hash)).await?;
		if state_nonce > nonce {
			info!(target: "nonce_search", "At block height {} and hash {:?} found nonce: {} which is greater than {} for Account address {}. Search is done", next_block_height, next_block_hash, state_nonce, nonce, account_id);
			return Ok(Some(BlockId::from((next_block_hash, next_block_height))));
		}

		info!(target: "nonce_search", "Looking for nonce > than: {} for Account address {}. At block height {} and hash {:?} found nonce: {}", nonce, account_id, next_block_height, next_block_hash, state_nonce);
		next_block_height += 1;
	}

	Ok(None)
} */
