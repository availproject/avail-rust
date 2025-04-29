use crate::{
	block::EventRecords,
	block_transaction::Filter,
	error::ClientError,
	from_substrate::{FeeDetails, RuntimeDispatchInfo},
	ABlock, AccountId, AvailConfig, AvailExtrinsicParamsBuilder, Client, H256,
};
use log::info;
use std::sync::Arc;
use std::time::Duration;
use subxt::config::Header;
use subxt::tx::Payload;
use subxt::{blocks::StaticExtrinsic, ext::scale_encode::EncodeAsFields, tx::DefaultPayload};
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

	pub async fn execute(&self, signer: &Keypair, options: Options) -> Result<SubmittedTransaction, subxt::Error> {
		self.client.sign_and_submit(signer, &self.payload, options).await
	}

	pub async fn execute_and_watch(
		&self,
		signer: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, SubmissionStateError> {
		self.client.sign_submit_and_watch(signer, &self.payload, options).await
	}
}

#[derive(Debug, Clone)]
pub struct TransactionDetails {
	client: Client,
	pub events: Option<Arc<EventRecords>>,
	pub tx_hash: H256,
	pub tx_index: u32,
	pub block_hash: H256,
	pub block_number: u32,
}

impl TransactionDetails {
	pub fn new(
		client: Client,
		events: Option<EventRecords>,
		tx_hash: H256,
		tx_index: u32,
		block_hash: H256,
		block_number: u32,
	) -> Self {
		let events = events.map(|x| x.into());
		Self {
			client,
			events,
			tx_hash,
			tx_index,
			block_hash,
			block_number,
		}
	}

	/// Returns None if it was not possible to determine if the transaction was successful or not
	/// If Some is returned then
	///    true means the transaction was successful
	///    false means the transaction failed
	pub fn is_successful(&self) -> Option<bool> {
		match &self.events {
			Some(events) => Some(events.has_system_extrinsic_success()),
			None => None,
		}
	}

	/// Returns Err if it was not possible to determine if the transaction was decodable
	/// If Ok is returned then
	///    Some means the transaction was successfully decoded
	///    None means the transaction cannot be decoded as T
	pub async fn decode_as<T: StaticExtrinsic + Clone>(&self) -> Result<Option<T>, ClientError> {
		let block = crate::block::Block::new(&self.client, self.block_hash).await?;
		let filter = Filter::new().tx_index(self.tx_index);
		let txs = block.transactions_static::<T>(filter);
		if txs.is_empty() {
			return Ok(None);
		}
		Ok(Some(txs[0].value.clone()))
	}

	/// Returns Err if it was not possible to determine if the transaction was decodable
	/// If Ok is returned then
	///    true means the transaction was successfully decoded
	///    false means the transaction cannot be decoded as T
	pub async fn is<T: StaticExtrinsic + Clone>(&self) -> Result<bool, ClientError> {
		let block = crate::block::Block::new(&self.client, self.block_hash).await?;
		let filter = Filter::new().tx_index(self.tx_index);
		let txs = block.transactions_static::<T>(filter);
		Ok(!txs.is_empty())
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
	pub fn new(client: Client, tx_hash: H256, account_id: AccountId, options: PopulatedOptions) -> Self {
		Self {
			client,
			tx_hash,
			account_id,
			options,
		}
	}

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

	pub async fn find_block_id_best_block(&self, sleep_duration: Duration) -> Result<Option<BlockId>, subxt::Error> {
		find_block_id_best_block(
			&self.client,
			&self.account_id,
			self.nonce(),
			self.mortality_period(),
			self.fork_height(),
			sleep_duration,
		)
		.await
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

/// TODO
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
}

#[derive(Debug, Clone, Copy)]
pub struct Options {
	pub app_id: Option<u32>,
	pub mortality: Option<u64>,
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

	pub fn mortality(mut self, value: u64) -> Self {
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

	pub async fn build(self, client: &Client, account_id: &AccountId) -> Result<PopulatedOptions, subxt::Error> {
		let app_id = self.app_id.unwrap_or_default();
		let tip = self.tip.unwrap_or_default();
		let nonce = match self.nonce {
			Some(x) => x as u64,
			None => client.rpc_system_account_next_index(account_id.to_string()).await? as u64,
		};
		let period = self.mortality.unwrap_or(32);
		let mortality = CheckedMortality::from_period(period, client).await?;

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
	pub mortality: CheckedMortality,
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
			self.mortality.block_number as u64,
			self.mortality.block_hash,
			self.mortality.period,
		);

		builder.build()
	}
}

#[derive(Debug, Clone, Copy)]
pub struct CheckedMortality {
	pub period: u64,
	pub block_hash: H256,
	pub block_number: u32,
}
impl CheckedMortality {
	pub fn new(period: u64, block_hash: H256, block_number: u32) -> Self {
		Self {
			period,
			block_hash,
			block_number,
		}
	}

	pub async fn from_period(period: u64, client: &Client) -> Result<Self, subxt::Error> {
		let finalized_hash = client.finalized_block_hash().await?;
		let header = client.header_at(finalized_hash).await?;
		let (block_hash, block_number) = (header.hash(), header.number());
		Ok(Self {
			period,
			block_hash,
			block_number,
		})
	}
}
