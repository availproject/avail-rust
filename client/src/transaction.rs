use super::platform::sleep;
use crate::{
	subxt_signer::sr25519::Keypair,
	transaction_options::{Options, RefinedMortality, RefinedOptions},
	Client,
};
use client_core::{
	avail::TransactionCallLike,
	config::TransactionLocation,
	rpc::substrate::BlockWithJustifications,
	transaction::{TransactionAdditional, TransactionCall},
	AccountId, BlockId, H256,
};
use std::{sync::Arc, time::Duration};
#[cfg(feature = "tracing")]
use tracing::info;

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

	pub async fn sign_and_submit(
		&self,
		signer: &Keypair,
		options: Options,
	) -> Result<SubmittedTransaction, client_core::Error> {
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

	pub async fn receipt(&self, use_best_block: bool) -> Result<Option<TransactionReceipt>, client_core::Error> {
		Utils::transaction_receipt(
			self.client.clone(),
			self.tx_hash,
			self.options.nonce,
			&self.account_id,
			&self.options.mortality,
			use_best_block,
		)
		.await
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BlockState {
	Included = 0,
	Finalized = 1,
	Discarded = 2,
	DoesNotExist = 3,
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

	pub async fn block_state(&self) -> Result<BlockState, client_core::Error> {
		self.client.block_state(self.block_id).await
	}

	pub async fn tx_events(&self) -> Result<(), ()> {
		unimplemented!()
	}
}

/// TODO
pub async fn get_new_or_cached_block(
	client: &Client,
	block_id: &BlockId,
) -> Result<Arc<BlockWithJustifications>, client_core::Error> {
	if let Some(block) = client.cache_client().find_signed_block(block_id.hash) {
		return Ok(block);
	}

	let block = client.block(block_id.hash).await?;
	let Some(block) = block else {
		let err = std::format!("{} not found", block_id.hash);
		return Err(err.into());
	};
	let block = Arc::new(block);
	if let Some(block) = client.cache_client().find_signed_block(block_id.hash) {
		return Ok(block);
	}
	client.cache_client().push_signed_block((block_id.hash, block.clone()));

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
		use_best_block: bool,
	) -> Result<Option<TransactionReceipt>, client_core::Error> {
		let Some(block_id) =
			Self::find_block_id_via_nonce(&client, nonce, account_id, mortality, use_best_block).await?
		else {
			return Ok(None);
		};

		let block = get_new_or_cached_block(&client, &block_id).await?;
		let Some(tx_location) = block.block.has_transaction(tx_hash) else {
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
	) -> Result<Option<BlockId>, client_core::Error> {
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
	) -> Result<Option<BlockId>, client_core::Error> {
		let mortality_ends_height = mortality.block_height + mortality.period as u32;

		let mut next_block_height = mortality.block_height + 1;
		let mut new_block_height = client.finalized_block_height().await?;

		#[cfg(feature = "tracing")]
		info!(target: "lib", "Nonce: {} Account address: {} Current Finalized Height: {} Mortality End Height: {}", nonce, account_id, new_block_height, mortality_ends_height);
		while mortality_ends_height >= next_block_height {
			if next_block_height > new_block_height {
				sleep(Duration::from_secs(3)).await;
				new_block_height = client.finalized_block_height().await?;
				continue;
			}

			let Some(next_block_hash) = client.block_hash(next_block_height).await? else {
				return Err(std::format!("Block hash not found. Height: {}", next_block_height).into());
			};

			let block_id = BlockId::from((next_block_hash, next_block_height));

			let state_nonce = client.block_nonce(account_id, next_block_hash).await?;
			if state_nonce > nonce {
				trace_new_block(nonce, state_nonce, account_id, block_id, true);
				return Ok(Some(BlockId::from((next_block_hash, next_block_height))));
			}

			trace_new_block(nonce, state_nonce, account_id, block_id, false);
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
	) -> Result<Option<BlockId>, client_core::Error> {
		let mortality_ends_height = mortality.block_height + mortality.period as u32;
		let mut current_block_id = BlockId::from((mortality.block_hash, mortality.block_height));
		let mut new_block_id = client.best_block_id().await?;

		#[cfg(feature = "tracing")]
		info!(target: "lib", "Nonce: {} Account address: {} Current Best Height: {} Mortality End Height: {}", nonce, account_id, new_block_id.height, mortality_ends_height);
		while mortality_ends_height >= current_block_id.height {
			if current_block_id.height > new_block_id.height || current_block_id.hash == new_block_id.hash {
				sleep(Duration::from_secs(3)).await;
				new_block_id = client.best_block_id().await?;
				continue;
			}

			if new_block_id.height == current_block_id.height || new_block_id.height == (current_block_id.height + 1) {
				let state_nonce = client.block_nonce(account_id, new_block_id.hash).await?;
				if state_nonce > nonce {
					trace_new_block(nonce, state_nonce, account_id, new_block_id, true);
					return Ok(Some(new_block_id));
				}
				trace_new_block(nonce, state_nonce, account_id, new_block_id, false);
				current_block_id = new_block_id;

				continue;
			}

			current_block_id.height += 1;
			let Some(hash) = client.block_hash(current_block_id.height).await? else {
				return Err(std::format!("Block hash not found. Height: {}", current_block_id.height).into());
			};
			current_block_id.hash = hash;

			let state_nonce = client.block_nonce(account_id, current_block_id.hash).await?;
			if state_nonce > nonce {
				trace_new_block(nonce, state_nonce, account_id, current_block_id, true);
				return Ok(Some(current_block_id));
			}

			trace_new_block(nonce, state_nonce, account_id, current_block_id, false);
		}

		Ok(None)
	}
}

#[cfg(feature = "tracing")]
fn trace_new_block(nonce: u32, state_nonce: u32, account_id: &AccountId, block_id: BlockId, search_done: bool) {
	if search_done {
		info!(target: "lib", "Account ({}, {}). At block ({}, {:?}) found nonce: {}. Search is done", nonce, account_id, block_id.height, block_id.hash, state_nonce);
	} else {
		info!(target: "lib", "Account ({}, {}). At block ({}, {:?}) found nonce: {}.", nonce, account_id, block_id.height, block_id.hash, state_nonce);
	}
}

#[cfg(not(feature = "tracing"))]
fn trace_new_block(_nonce: u32, _state_nonce: u32, _account_id: &AccountId, _block_id: BlockId, _search_done: bool) {
	return;
}
