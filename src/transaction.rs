use super::platform::sleep;
use crate::{
	api_dev_custom::TransactionCallLike,
	client::{rpc::ChainBlock, Client},
	config::*,
	error::RpcError,
	primitives::transaction::{TransactionAdditional, TransactionCall},
	transaction_options::{Options, RefinedMortality, RefinedOptions},
};
use log::info;
use primitive_types::H256;
use std::{sync::Arc, time::Duration};
use subxt_signer::sr25519::Keypair;

pub trait SubmittableTransactionLike {
	fn to_submittable(&self, client: Client) -> SubmittableTransaction;
}

impl<T: TransactionCallLike> SubmittableTransactionLike for T {
	fn to_submittable(&self, client: Client) -> SubmittableTransaction {
		let call = self.to_call();
		SubmittableTransaction::new(client, call)
	}
}

#[derive(Clone)]
pub struct SubmittableTransaction {
	client: Client,
	pub call: TransactionCall,
}

impl SubmittableTransaction {
	pub fn new(client: Client, call: TransactionCall) -> Self {
		{
			Self { client, call }
		}
	}

	pub async fn sign_and_submit(&self, signer: &Keypair, options: Options) -> Result<SubmittedTransaction, RpcError> {
		self.client.sign_and_submit_call(signer, &self.call, options).await
	}
}

#[derive(Clone, Copy)]
pub enum ReceiptMethod {
	Default { use_best_block: bool },
}

impl Default for ReceiptMethod {
	fn default() -> Self {
		Self::Default { use_best_block: false }
	}
}

#[derive(Clone)]
pub struct SubmittedTransaction {
	client: Client,
	pub tx_hash: H256,
	pub account_id: AccountId,
	pub options: RefinedOptions,
	pub additional: TransactionAdditional,
}

impl SubmittedTransaction {
	pub fn new(
		client: Client,
		tx_hash: H256,
		account_id: AccountId,
		options: RefinedOptions,
		additional: TransactionAdditional,
	) -> Self {
		Self {
			client,
			tx_hash,
			account_id,
			options,
			additional,
		}
	}

	pub async fn receipt(&self, method: ReceiptMethod) -> Result<Option<TransactionReceipt>, RpcError> {
		Utils::transaction_receipt(
			self.client.clone(),
			self.tx_hash,
			self.options.nonce,
			&self.account_id,
			&self.options.mortality,
			&method,
		)
		.await
	}

	pub async fn transaction_overview(&self) -> Result<(), RpcError> {
		unimplemented!()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum BlockState {
	Included,
	Finalized,
	Discarded,
	DoesNotExist,
}

#[derive(Clone)]
pub struct TransactionReceipt {
	client: Client,
	pub block_id: BlockId,
	pub tx_location: TransactionLocation,
}

impl TransactionReceipt {
	pub fn new(client: Client, block_id: BlockId, tx_location: TransactionLocation) -> Self {
		Self {
			client,
			block_id,
			tx_location,
		}
	}

	pub async fn block_state(&self) -> Result<BlockState, RpcError> {
		self.client.block_state(self.block_id).await
	}

	pub async fn tx_events(&self) -> Result<(), ()> {
		unimplemented!()
	}
}

/// TODO
pub async fn get_new_or_cached_block(client: &Client, block_id: &BlockId) -> Result<Arc<ChainBlock>, RpcError> {
	if let Ok(cache) = client.cache.lock() {
		if let Some(block) = cache.chain_blocks_cache.find(block_id.hash) {
			return Ok(block);
		}
	}

	let block = client.block(block_id.hash).await?;
	let Some(block) = block else {
		let err = std::format!("{} not found", block_id.hash);
		return Err(err.into());
	};
	let block = Arc::new(block);
	if let Ok(mut cache) = client.cache.lock() {
		if let Some(block) = cache.chain_blocks_cache.find(block_id.hash) {
			return Ok(block);
		}
		cache.chain_blocks_cache.push((block_id.hash, block.clone()));
	}

	Ok(block)
}

pub struct Utils;
impl Utils {
	/// TODO
	pub async fn transaction_receipt(
		client: Client,
		tx_hash: H256,
		nonce: u32,
		account_id: &AccountId,
		mortality: &RefinedMortality,
		method: &ReceiptMethod,
	) -> Result<Option<TransactionReceipt>, RpcError> {
		let ReceiptMethod::Default { use_best_block } = *method;
		let Some(block_id) =
			Self::find_block_id_via_nonce(&client, nonce, account_id, mortality, use_best_block).await?
		else {
			return Ok(None);
		};

		let block = get_new_or_cached_block(&client, &block_id).await?;
		let Some(tx_location) = block.has_transaction(tx_hash) else {
			return Ok(None);
		};

		Ok(Some(TransactionReceipt::new(client, block_id, tx_location)))
	}

	/// TODO
	pub async fn find_block_id_via_nonce(
		client: &Client,
		nonce: u32,
		account_id: &AccountId,
		mortality: &RefinedMortality,
		use_best_block: bool,
	) -> Result<Option<BlockId>, RpcError> {
		match use_best_block {
			true => Self::find_best_block_block_id_via_nonce(client, nonce, account_id, mortality).await,
			false => Self::find_finalized_block_block_id_via_nonce(client, nonce, account_id, mortality).await,
		}
	}

	/// TODO
	pub async fn find_finalized_block_block_id_via_nonce(
		client: &Client,
		nonce: u32,
		account_id: &AccountId,
		mortality: &RefinedMortality,
	) -> Result<Option<BlockId>, RpcError> {
		let mortality_ends_height = mortality.block_height + mortality.period as u32;

		let mut next_block_height = mortality.block_height + 1;
		let mut block_height = client.finalized_block_height().await?;

		info!(target: "nonce_search", "Nonce: {} Account address: {} Current Finalized Height: {} Mortality End Height: {}", nonce, account_id, block_height, mortality_ends_height);
		while mortality_ends_height >= next_block_height {
			if next_block_height > block_height {
				sleep(Duration::from_secs(3)).await;
				block_height = client.finalized_block_height().await?;
				continue;
			}

			let Some(next_block_hash) = client.block_hash(next_block_height).await? else {
				return Err(std::format!("Block hash not found. Height: {}", next_block_height).into());
			};

			let state_nonce = client.nonce_state(account_id, next_block_hash).await?;
			if state_nonce > nonce {
				info!(target: "nonce_search", "Account ({}, {}). At block ({}, {:?}) found nonce: {}. Search is done.", nonce, account_id, next_block_height, next_block_hash, state_nonce);
				return Ok(Some(BlockId::from((next_block_hash, next_block_height))));
			}

			info!(target: "nonce_search", "Account ({}, {}). At block ({}, {:?}) found nonce: {}", nonce, account_id, next_block_height, next_block_hash, state_nonce);
			next_block_height += 1;
		}

		Ok(None)
	}

	/// TODO
	pub async fn find_best_block_block_id_via_nonce(
		client: &Client,
		nonce: u32,
		account_id: &AccountId,
		mortality: &RefinedMortality,
	) -> Result<Option<BlockId>, RpcError> {
		let mortality_ends_height = mortality.block_height + mortality.period as u32;

		let mut next_block_height = mortality.block_height + 1;
		let mut next_block_hash = H256::zero();
		let mut block_id = client.best_block_id().await?;

		info!(target: "nonce_search", "Nonce: {} Account address: {} Current Best Height: {} Mortality End Height: {}", nonce, account_id, block_id.height, mortality_ends_height);
		while mortality_ends_height >= next_block_height {
			if next_block_hash == block_id.hash || next_block_height > block_id.height {
				sleep(Duration::from_secs(3)).await;
				block_id = client.best_block_id().await?;
				continue;
			}

			if next_block_height == (block_id.height + 1) {
				next_block_hash = block_id.hash;
				let state_nonce = client.nonce_state(account_id, next_block_hash).await?;
				if state_nonce > nonce {
					info!(target: "nonce_search", "Account ({}, {}). At block ({}, {:?}) found nonce: {}. Search is done.", nonce, account_id, next_block_height, next_block_hash, state_nonce);
					return Ok(Some(BlockId::from((next_block_hash, next_block_height))));
				}
				info!(target: "nonce_search", "Account ({}, {}). At block ({}, {:?})found nonce: {}", nonce, account_id, next_block_height, next_block_hash, state_nonce);
			} else {
				let Some(hash) = client.block_hash(next_block_height).await? else {
					return Err(std::format!("Block hash not found. Height: {}", next_block_height).into());
				};

				next_block_hash = hash;
				let state_nonce = client.nonce_state(account_id, next_block_hash).await?;
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
}
