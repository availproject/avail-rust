use super::{Options, TransactionDetails};
use crate::{account, block::EventRecords, rpc, ABlock, AccountId, Client, WaitFor, H256};
use log::info;
use std::time::Duration;
use subxt::{blocks::StaticExtrinsic, ext::scale_encode::EncodeAsFields, tx::DefaultPayload};
use subxt_signer::sr25519::Keypair;

use super::{options::CheckedMortality, PopulatedOptions};

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
	signer: &Keypair,
	call: &DefaultPayload<T>,
	options: Options,
) -> Result<H256, subxt::Error>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = signer.public_key().to_account_id();
	let options = options.build(client, &account_id).await?;
	let params = options.build().await;

	if params.6 .0 .0 != 0 && (call.pallet_name() != "DataAvailability" || call.call_name() != "submit_data") {
		return Err(subxt::Error::Other(
			"Transaction is not compatible with non-zero AppIds".into(),
		));
	}

	let tx_client = client.online_client.tx();
	let signed_call = tx_client.create_signed(call, signer, params).await?;
	let extrinsic = signed_call.encoded();
	let tx_hash = rpc::author::submit_extrinsic(client, extrinsic).await?;
	info!(target: "submission", "Transaction submitted. Tx Hash: {:?}, Fork Hash: {:?}, Fork Height: {:?}, Period: {}, Nonce: {}, Account Address: {}", tx_hash, options.mortality.block_hash, options.mortality.block_number, options.mortality.period, options.nonce, account_id);

	Ok(tx_hash)
}

pub async fn sign_and_send_v2<T>(
	client: &Client,
	signer: &Keypair,
	call: &DefaultPayload<T>,
	options: Options,
) -> Result<SubmittedTransaction, subxt::Error>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let account_id = signer.public_key().to_account_id();
	let options = options.build(client, &account_id).await?;
	let params = options.build().await;

	if params.6 .0 .0 != 0 && (call.pallet_name() != "DataAvailability" || call.call_name() != "submit_data") {
		return Err(subxt::Error::Other(
			"Transaction is not compatible with non-zero AppIds".into(),
		));
	}

	let tx_client = client.online_client.tx();
	let signed_call = tx_client.create_signed(call, signer, params).await?;
	let extrinsic = signed_call.encoded();
	let tx_hash = rpc::author::submit_extrinsic(client, extrinsic).await?;
	info!(target: "submission", "Transaction submitted. Tx Hash: {:?}, Fork Hash: {:?}, Fork Height: {:?}, Period: {}, Nonce: {}, Account Address: {}", tx_hash, options.mortality.block_hash, options.mortality.block_number, options.mortality.period, options.nonce, account_id);

	let st = SubmittedTransaction::new(client.clone(), tx_hash, account_id, &options);

	Ok(st)
}

pub async fn sign_send_and_watch<T>(
	client: &Client,
	signer: &Keypair,
	call: &DefaultPayload<T>,
	wait_for: WaitFor,
	options: Options,
) -> Result<Option<TransactionDetails>, subxt::Error>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	let st = sign_and_send_v2(client, signer, call, options).await?;
	let method = ReceiptMethod::Default {
		use_best_block: wait_for == WaitFor::BlockInclusion,
		sleep_duration: Duration::from_secs(5),
	};

	let mut retry = 1u32;
	let (block, block_id) = loop {
		let maybe_block_id = transaction_maybe_block_id(client, &st.tx_extra, &method).await?;
		let Some(block_id) = maybe_block_id else {
			return Ok(None);
		};

		let block = client.block_at(block_id.hash).await;
		let block = match block {
			Ok(x) => x,
			Err(err) => {
				if retry > 0 {
					retry -= 1;
					continue;
				}
				return Err(err);
			},
		};

		break (block, block_id);
	};

	find_transaction(client, st.tx_hash, block_id, &block).await
}

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

#[derive(Clone, Copy)]
pub enum ReceiptMethod {
	Default {
		sleep_duration: Duration,
		use_best_block: bool,
	},
}

impl Default for ReceiptMethod {
	fn default() -> Self {
		Self::Default {
			sleep_duration: Duration::from_secs(5),
			use_best_block: false,
		}
	}
}

#[derive(Clone)]
pub struct SubmittedTransaction {
	_client: Client,
	pub tx_hash: H256,
	pub tx_extra: TransactionExtra,
}

impl SubmittedTransaction {
	pub fn new(client: Client, tx_hash: H256, account_id: AccountId, options: &PopulatedOptions) -> Self {
		let tx_extra = TransactionExtra {
			account_id,
			nonce: options.nonce as u32,
			app_id: options.app_id,
			tip: options.tip,
			mortality: options.mortality,
		};
		Self {
			_client: client,
			tx_hash,
			tx_extra,
		}
	}
}

#[derive(Clone)]
pub struct TransactionExtra {
	pub account_id: AccountId,
	pub nonce: u32,
	pub app_id: u32,
	pub tip: u128,
	pub mortality: CheckedMortality,
}

pub async fn find_transaction(
	client: &Client,
	tx_hash: H256,
	block_id: BlockId,
	block: &ABlock,
) -> Result<Option<TransactionDetails>, subxt::Error> {
	let exts = block.extrinsics().await?;
	for (tx_index, tx) in exts.iter().enumerate() {
		if tx.hash() == tx_hash {
			let events = tx.events().await.ok();
			let events = events.and_then(EventRecords::new_ext);
			let value = TransactionDetails::new(
				client.clone(),
				events,
				tx_hash,
				tx_index as u32,
				block_id.hash,
				block_id.height,
			);
			return Ok(Some(value));
		}
	}

	return Ok(None);
}

pub async fn transaction_maybe_block_id(
	client: &Client,
	tx_extra: &TransactionExtra,
	method: &ReceiptMethod,
) -> Result<Option<BlockId>, subxt::Error> {
	match method {
		ReceiptMethod::Default {
			use_best_block,
			sleep_duration,
		} => match use_best_block {
			true => transaction_maybe_block_id_best_block(client, tx_extra, *sleep_duration).await,
			false => transaction_maybe_block_id_finalized_block(client, tx_extra, *sleep_duration).await,
		},
	}
}

pub async fn transaction_maybe_block_id_finalized_block(
	client: &Client,
	tx_extra: &TransactionExtra,
	sleep_duration: Duration,
) -> Result<Option<BlockId>, subxt::Error> {
	let (nonce, account_id, mortality) = (tx_extra.nonce, &tx_extra.account_id, &tx_extra.mortality);
	let mortality_ends_height = mortality.block_number + mortality.period as u32;
	let address = std::format!("{}", tx_extra.account_id);

	let mut next_block_height = mortality.block_number + 1;
	let mut block_height = client.finalized_block_number().await?;

	info!(target: "nonce_search", "Nonce: {} Account address: {} Current Finalized Height: {} Mortality End Height: {}", nonce, account_id, block_height, mortality_ends_height);
	while mortality_ends_height >= next_block_height {
		if next_block_height > block_height {
			tokio::time::sleep(sleep_duration).await;
			block_height = client.finalized_block_number().await?;
			continue;
		}

		let Some(next_block_hash) = client.block_hash(next_block_height).await? else {
			let err = std::format!("{}", next_block_height);
			let err = subxt::Error::Block(subxt::error::BlockError::NotFound(err));
			return Err(err);
		};

		let state_nonce = account::nonce_state(client, &address, Some(next_block_hash)).await?;
		if state_nonce > nonce {
			info!(target: "nonce_search", "Account ({}, {}). At block ({}, {:?}) found nonce: {}. Search is done.", nonce, account_id, next_block_height, next_block_hash, state_nonce);
			return Ok(Some(BlockId::from((next_block_hash, next_block_height))));
		}

		info!(target: "nonce_search", "Account ({}, {}). At block ({}, {:?}) found nonce: {}", nonce, account_id, next_block_height, next_block_hash, state_nonce);
		next_block_height += 1;
	}

	Ok(None)
}

pub async fn transaction_maybe_block_id_best_block(
	client: &Client,
	tx_extra: &TransactionExtra,
	sleep_duration: Duration,
) -> Result<Option<BlockId>, subxt::Error> {
	let (nonce, account_id, mortality) = (tx_extra.nonce, &tx_extra.account_id, &tx_extra.mortality);
	let mortality_ends_height = mortality.block_number + mortality.period as u32;
	let address = std::format!("{}", tx_extra.account_id);

	let mut next_block_height = mortality.block_number + 1;
	let mut next_block_hash = H256::zero();
	let mut block_id = client.best_block_id().await?;

	info!(target: "nonce_search", "Nonce: {} Account address: {} Current Best Height: {} Mortality End Height: {}", nonce, account_id, block_id.height, mortality_ends_height);
	while mortality_ends_height >= next_block_height {
		if next_block_hash == block_id.hash || next_block_height > block_id.height {
			tokio::time::sleep(sleep_duration).await;
			block_id = client.best_block_id().await?;
			continue;
		}

		if next_block_height == (block_id.height + 1) {
			next_block_hash = block_id.hash;
			let state_nonce = account::nonce_state(client, &address, Some(next_block_hash)).await?;
			if state_nonce > nonce {
				info!(target: "nonce_search", "Account ({}, {}). At block ({}, {:?}) found nonce: {}. Search is done.", nonce, account_id, next_block_height, next_block_hash, state_nonce);
				return Ok(Some(BlockId::from((next_block_hash, next_block_height))));
			}
			info!(target: "nonce_search", "Account ({}, {}). At block ({}, {:?})found nonce: {}", nonce, account_id, next_block_height, next_block_hash, state_nonce);
		} else {
			let Some(hash) = client.block_hash(next_block_height).await? else {
				let err = std::format!("{}", next_block_height);
				let err = subxt::Error::Block(subxt::error::BlockError::NotFound(err));
				return Err(err);
			};

			next_block_hash = hash;
			let state_nonce = account::nonce_state(client, &address, Some(next_block_hash)).await?;
			if state_nonce > nonce {
				info!(target: "nonce_search", "Account ({}, {}). At block ({}, {:?}) found nonce: {}. Search is done.", nonce, account_id, next_block_height, next_block_hash, state_nonce);
				return Ok(Some(BlockId::from((next_block_hash, next_block_height))));
			}
			info!(target: "nonce_search", "Account ({}, {}). At block ({}, {:?}) found nonce: {}", nonce, account_id, next_block_height, next_block_hash, state_nonce);
			next_block_height += 1;
		}
	}

	Ok(None)
}
