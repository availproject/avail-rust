use crate::{
	client_rpc::ChainBlock,
	error::{ClientError, RpcError},
	from_substrate::{FeeDetails, RuntimeDispatchInfo},
	AccountId, AvailConfig, AvailExtrinsicParamsBuilder, Client, H256,
};
use log::info;
use std::{sync::Arc, time::Duration};
use subxt::{
	blocks::StaticExtrinsic,
	config::Header,
	ext::scale_encode::EncodeAsFields,
	tx::{DefaultPayload, Payload},
};
use subxt_signer::sr25519::Keypair;

pub type Params =
	<<AvailConfig as subxt::Config>::ExtrinsicParams as subxt::config::ExtrinsicParams<AvailConfig>>::Params;

#[derive(Debug, Clone)]
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
	) -> Result<RuntimeDispatchInfo, ClientError> {
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
	) -> Result<FeeDetails, ClientError> {
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

	pub async fn payment_query_call_info(&self) -> Result<RuntimeDispatchInfo, ClientError> {
		let metadata = self.client.online_client.metadata();
		let call = self.payload.encode_call_data(&metadata)?;

		self.client.api_transaction_payment_query_call_info(call, None).await
	}

	pub async fn payment_query_call_fee_details(&self) -> Result<FeeDetails, ClientError> {
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
		use_best_block: bool,
	},
	Blocks {
		sleep_duration: Duration,
		use_best_block: bool,
	},
	RPCTransactionOverview {
		sleep_duration: Duration,
		use_best_block: bool,
	},
}

impl Default for ReceiptMethod {
	fn default() -> Self {
		Self::Default { use_best_block: false }
	}
}

#[derive(Clone)]
pub struct TransactionLocation {
	pub hash: H256,
	pub index: u32,
}

impl From<(H256, u32)> for TransactionLocation {
	fn from(value: (H256, u32)) -> Self {
		Self {
			hash: value.0,
			index: value.1,
		}
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
		transaction_receipt(self.client.clone(), self.tx_hash, self.tx_extra.clone(), &method).await
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

pub async fn transaction_receipt(
	client: Client,
	tx_hash: H256,
	tx_extra: TransactionExtra,
	method: &ReceiptMethod,
) -> Result<Option<TransactionReceipt>, RpcError> {
	let Some(block_id) = transaction_maybe_block_id(&client, &tx_extra, method).await? else {
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

/// TODO
pub async fn transaction_maybe_block_id(
	client: &Client,
	tx_extra: &TransactionExtra,
	method: &ReceiptMethod,
) -> Result<Option<BlockId>, RpcError> {
	match method {
		ReceiptMethod::Default { use_best_block } => match use_best_block {
			true => transaction_maybe_block_id_best_block(client, tx_extra).await,
			false => transaction_maybe_block_id_finalized_block(client, tx_extra).await,
		},
		_ => unimplemented!(),
	}
}

/// TODO
pub async fn transaction_maybe_block_id_finalized_block(
	client: &Client,
	tx_extra: &TransactionExtra,
) -> Result<Option<BlockId>, RpcError> {
	let (nonce, account_id, mortality) = (tx_extra.nonce, &tx_extra.account_id, &tx_extra.mortality);
	let mortality_ends_height = mortality.block_height + mortality.period as u32;

	let mut next_block_height = mortality.block_height + 1;
	let mut block_height = client.finalized_block_height().await?;

	info!(target: "nonce_search", "Nonce: {} Account address: {} Current Finalized Height: {} Mortality End Height: {}", nonce, account_id, block_height, mortality_ends_height);
	while mortality_ends_height >= next_block_height {
		if next_block_height > block_height {
			tokio::time::sleep(Duration::from_secs(3)).await;
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
pub async fn transaction_maybe_block_id_best_block(
	client: &Client,
	tx_extra: &TransactionExtra,
) -> Result<Option<BlockId>, RpcError> {
	let (nonce, account_id, mortality) = (tx_extra.nonce, &tx_extra.account_id, &tx_extra.mortality);
	let mortality_ends_height = mortality.block_height + mortality.period as u32;

	let mut next_block_height = mortality.block_height + 1;
	let mut next_block_hash = H256::zero();
	let mut block_id = client.best_block_id().await?;

	info!(target: "nonce_search", "Nonce: {} Account address: {} Current Best Height: {} Mortality End Height: {}", nonce, account_id, block_id.height, mortality_ends_height);
	while mortality_ends_height >= next_block_height {
		if next_block_hash == block_id.hash || next_block_height > block_id.height {
			tokio::time::sleep(Duration::from_secs(3)).await;
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

#[derive(Debug, Clone, Copy)]
pub struct Options {
	pub app_id: Option<u32>,
	pub mortality: Option<MortalityOption>,
	pub nonce: Option<u32>,
	pub tip: Option<u128>,
}

impl Options {
	pub fn new() -> Self {
		Self {
			app_id: None,
			mortality: None,
			nonce: None,
			tip: None,
		}
	}

	pub fn app_id(mut self, value: u32) -> Self {
		self.app_id = Some(value);
		self
	}

	pub fn mortality(mut self, value: MortalityOption) -> Self {
		self.mortality = Some(value);
		self
	}

	pub fn nonce(mut self, value: u32) -> Self {
		self.nonce = Some(value);
		self
	}

	pub fn tip(mut self, value: u128) -> Self {
		self.tip = Some(value);
		self
	}

	pub async fn build(self, client: &Client, account_id: &AccountId) -> Result<PopulatedOptions, RpcError> {
		let app_id = self.app_id.unwrap_or_default();
		let tip = self.tip.unwrap_or_default();
		let nonce = match self.nonce {
			Some(x) => x as u64,
			None => client.rpc_system_account_next_index(&account_id.to_string()).await? as u64,
		};
		let mortality = match &self.mortality {
			Some(MortalityOption::Period(period)) => Mortality::from_period(client, *period).await?,
			Some(MortalityOption::Full(mortality)) => mortality.clone(),
			None => Mortality::from_period(client, 32).await?,
		};

		Ok(PopulatedOptions {
			app_id,
			mortality,
			nonce,
			tip,
		})
	}
}

impl Default for Options {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone)]
pub struct PopulatedOptions {
	pub app_id: u32,
	pub mortality: Mortality,
	pub nonce: u64,
	pub tip: u128,
}

impl PopulatedOptions {
	pub async fn build(self) -> Params {
		let mut builder = AvailExtrinsicParamsBuilder::new();
		builder = builder.app_id(self.app_id);
		builder = builder.tip(self.tip);
		builder = builder.nonce(self.nonce);

		builder = builder.mortal_unchecked(
			self.mortality.block_height as u64,
			self.mortality.block_hash,
			self.mortality.period,
		);

		builder.build()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum MortalityOption {
	Period(u64),
	Full(Mortality),
}

#[derive(Debug, Clone, Copy)]
pub struct Mortality {
	pub period: u64,
	pub block_hash: H256,
	pub block_height: u32,
}
impl Mortality {
	pub fn new(period: u64, block_hash: H256, block_height: u32) -> Self {
		Self {
			period,
			block_hash,
			block_height,
		}
	}

	pub async fn from_period(client: &Client, period: u64) -> Result<Self, RpcError> {
		let header = client.finalized_block_header().await?;
		let (block_hash, block_height) = (header.hash(), header.number());
		Ok(Self {
			period,
			block_hash,
			block_height,
		})
	}
}
