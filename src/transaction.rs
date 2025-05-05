use super::platform::sleep;
use crate::{
	client::{rpc::ChainBlock, Client},
	config::*,
	error::RpcError,
	from_substrate::{FeeDetails, RuntimeDispatchInfo},
	transaction_options::{Mortality, Options, PopulatedOptions},
};
use log::info;
use primitive_types::H256;
use std::{sync::Arc, time::Duration};
use subxt::{
	blocks::StaticExtrinsic,
	ext::scale_encode::EncodeAsFields,
	tx::{DefaultPayload, Payload},
};
use subxt_signer::sr25519::Keypair;

#[derive(Clone)]
pub struct SubmittableTransaction<T>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	client: Client,
	payload: DefaultPayload<T>,
}

impl<T> SubmittableTransaction<T>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	pub fn new(client: Client, payload: DefaultPayload<T>) -> Self {
		Self { client, payload }
	}

	pub async fn payment_query_info(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<RuntimeDispatchInfo, RpcError> {
		let account_id = account.public_key().to_account_id();
		let options = options.unwrap_or_default().build(&self.client, &account_id).await?;

		let params = options.build().await;
		let tx = self
			.client
			.online_client
			.tx()
			.create_signed(&self.payload, account, params)
			.await?;

		let tx = tx.encoded();

		self.client.api_transaction_payment_query_info(tx.to_vec(), None).await
	}

	pub async fn payment_query_fee_details(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<FeeDetails, RpcError> {
		let account_id = account.public_key().to_account_id();
		let options = options.unwrap_or_default().build(&self.client, &account_id).await?;

		let params = options.build().await;
		let tx = self
			.client
			.online_client
			.tx()
			.create_signed(&self.payload, account, params)
			.await?;

		let tx = tx.encoded();

		self.client
			.api_transaction_payment_query_fee_details(tx.to_vec(), None)
			.await
	}

	pub async fn payment_query_call_info(&self) -> Result<RuntimeDispatchInfo, RpcError> {
		let metadata = self.client.online_client.metadata();
		let call = self.payload.encode_call_data(&metadata)?;

		self.client.api_transaction_payment_query_call_info(call, None).await
	}

	pub async fn payment_query_call_fee_details(&self) -> Result<FeeDetails, RpcError> {
		let metadata = self.client.online_client.metadata();
		let call = self.payload.encode_call_data(&metadata)?;

		self.client
			.api_transaction_payment_query_call_fee_details(call, None)
			.await
	}

	pub async fn sign_and_submit(&self, signer: &Keypair, options: Options) -> Result<SubmittedTransaction, RpcError> {
		self.client.sign_and_submit(signer, &self.payload, options).await
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
	pub tx_extra: TransactionExtra,
}

impl SubmittedTransaction {
	pub fn new(client: Client, tx_hash: H256, account_id: AccountId, options: &PopulatedOptions) -> Self {
		let tx_extra = TransactionExtra {
			account_id,
			nonce: options.nonce as u32,
			app_id: options.app_id,
			tip: options.tip,
			mortality: options.mortality.clone(),
		};
		Self {
			client,
			tx_hash,
			tx_extra,
		}
	}

	pub async fn receipt(&self, method: ReceiptMethod) -> Result<Option<TransactionReceipt>, RpcError> {
		Utils::transaction_receipt(self.client.clone(), self.tx_hash, self.tx_extra.clone(), &method).await
	}

	pub async fn transaction_overview(&self) -> Result<(), RpcError> {
		unimplemented!()
	}
}

#[derive(Clone)]
pub struct TransactionExtra {
	pub account_id: AccountId,
	pub nonce: u32,
	pub app_id: u32,
	pub tip: u128,
	pub mortality: Mortality,
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
	pub tx_extra: TransactionExtra,
}

impl TransactionReceipt {
	pub fn new(
		client: Client,
		block_id: BlockId,
		tx_location: TransactionLocation,
		tx_extra: TransactionExtra,
	) -> Self {
		Self {
			client,
			block_id,
			tx_location,
			tx_extra,
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
pub async fn get_new_or_cached_block(client: &Client, block_id: &BlockId) -> Result<Arc<ChainBlock>, subxt::Error> {
	if let Ok(cache) = client.cache.lock() {
		if let Some(block) = cache.chain_blocks_cache.find(block_id.hash) {
			return Ok(block);
		}
	}

	let block = client.block(block_id.hash).await?;
	let Some(block) = block else {
		let err = std::format!("{}", block_id.hash);
		let err = subxt::Error::Block(subxt::error::BlockError::NotFound(err));
		return Err(err);
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
		tx_extra: TransactionExtra,
		method: &ReceiptMethod,
	) -> Result<Option<TransactionReceipt>, RpcError> {
		let ReceiptMethod::Default { use_best_block } = *method;
		let Some(block_id) = Self::find_block_id_via_nonce(&client, &tx_extra, use_best_block).await? else {
			return Ok(None);
		};

		let block = get_new_or_cached_block(&client, &block_id).await?;
		let Some(tx_location) = block.has_transaction(tx_hash) else {
			return Ok(None);
		};

		let receipt = TransactionReceipt {
			client,
			block_id,
			tx_location,
			tx_extra,
		};

		Ok(Some(receipt))
	}

	/// TODO
	pub async fn find_block_id_via_nonce(
		client: &Client,
		tx_extra: &TransactionExtra,
		use_best_block: bool,
	) -> Result<Option<BlockId>, RpcError> {
		let (nonce, account_id, mortality) = (tx_extra.nonce, &tx_extra.account_id, &tx_extra.mortality);
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
		mortality: &Mortality,
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

			let state_nonce = client.nonce_state(&account_id, next_block_hash).await?;
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
		mortality: &Mortality,
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
				let state_nonce = client.nonce_state(&account_id, next_block_hash).await?;
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
				let state_nonce = client.nonce_state(&account_id, next_block_hash).await?;
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
