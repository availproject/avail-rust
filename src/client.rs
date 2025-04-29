use crate::{
	avail::{runtime_types::pallet_balances::types::AccountData, system::storage::types::account::Account},
	block::EventRecords,
	client_rpc::ChainBlock,
	error::ClientError,
	transaction::{find_transaction, SubmissionStateError, SubmittedTransaction},
	ABlock, ABlocksClient, AConstantsClient, AEventsClient, AOnlineClient, AStorageClient, AccountId, AccountIdExt,
	AvailHeader, Options, TransactionDetails, H256,
};
use log::info;
use std::{fmt::Debug, sync::Arc, time::Duration};
use subxt::{
	backend::rpc::{
		reconnecting_rpc_client::{ExponentialBackoff, RpcClient as ReconnectingRpcClient},
		RpcClient,
	},
	blocks::StaticExtrinsic,
	ext::scale_encode::EncodeAsFields,
	tx::DefaultPayload,
};
use subxt_signer::sr25519::Keypair;

#[cfg(feature = "native")]
use crate::http;

pub async fn reconnecting_api(endpoint: &str) -> Result<Client, ClientError> {
	let rpc_client = ReconnectingRpcClient::builder()
		.max_request_size(512 * 1024 * 1024)
		.max_response_size(512 * 1024 * 1024)
		.retry_policy(
			ExponentialBackoff::from_millis(1000)
				.max_delay(Duration::from_secs(3))
				.take(3),
		)
		.build(endpoint)
		.await
		.map_err(|e| e.to_string())?;
	let rpc_client = RpcClient::new(rpc_client);

	// Cloning RpcClient is cheaper and doesn't create a new WS connection
	let api = AOnlineClient::from_rpc_client(rpc_client.clone()).await?;

	Ok(Client::new(api, rpc_client))
}

#[cfg(feature = "native")]
pub async fn http_api(endpoint: &str) -> Result<Client, ClientError> {
	let rpc_client = http::HttpClient::new(endpoint).map_err(|e| e.to_string())?;
	let rpc_client = RpcClient::new(rpc_client);

	// Cloning RpcClient is cheaper and doesn't create a new WS connection
	let api = AOnlineClient::from_rpc_client(rpc_client.clone()).await?;
	let client = Client::new(api, rpc_client);

	Ok(client)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ClientOptions {
	pub tx_state_rpc_enabled: bool,
}

type SharedClientOptions = Arc<std::sync::Mutex<ClientOptions>>;
type SharedCache = Arc<std::sync::Mutex<Cache>>;

#[derive(Default)]
pub struct Cache {
	pub last_fetched_block: Option<(H256, Arc<ABlock>)>,
}

impl Debug for Cache {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Cache").field("last_fetched_block", &"").finish()
	}
}

#[derive(Debug, Clone)]
pub struct Client {
	pub online_client: AOnlineClient,
	pub rpc_client: RpcClient,
	pub options: SharedClientOptions,
	pub cache: SharedCache,
}

impl Client {
	pub fn new(online_client: AOnlineClient, rpc_client: RpcClient) -> Client {
		Self {
			online_client,
			rpc_client,
			options: SharedClientOptions::default(),
			cache: SharedCache::default(),
		}
	}

	pub fn get_options(&self) -> ClientOptions {
		if let Ok(lock) = self.options.lock() {
			return *lock;
		}
		ClientOptions::default()
	}

	pub async fn event_records(&self, at: H256) -> Result<Option<EventRecords>, subxt::Error> {
		let events = self.subxt_events().at(at).await?;
		Ok(EventRecords::new(events))
	}

	pub async fn block_at(&self, at: H256) -> Result<ABlock, subxt::Error> {
		self.online_client.blocks().at(at).await
	}

	pub async fn block_transaction_at(
		&self,
		tx_hash: H256,
		at: H256,
	) -> Result<Option<TransactionDetails>, subxt::Error> {
		let block = self.online_client.blocks().at(at).await?;
		super::transaction::find_transaction(self, &block, &tx_hash).await
	}

	// Header
	pub async fn header(&self, at: H256) -> Result<AvailHeader, subxt::Error> {
		self.rpc_chain_get_header(Some(at)).await
	}

	pub async fn best_block_header(&self) -> Result<AvailHeader, subxt::Error> {
		self.header(self.best_block_hash().await?).await
	}

	pub async fn finalized_block_header(&self) -> Result<AvailHeader, subxt::Error> {
		self.header(self.finalized_block_hash().await?).await
	}

	// (RPC) Block
	pub async fn block(&self, at: H256) -> Result<ChainBlock, subxt::Error> {
		self.rpc_chain_get_block(Some(at)).await
	}

	pub async fn best_block(&self) -> Result<ChainBlock, subxt::Error> {
		self.block(self.best_block_hash().await?).await
	}

	pub async fn finalized_block(&self) -> Result<ChainBlock, subxt::Error> {
		self.block(self.best_block_hash().await?).await
	}

	// Block Hash
	pub async fn block_hash(&self, block_height: u32) -> Result<H256, subxt::Error> {
		self.rpc_chain_get_block_hash(Some(block_height)).await
	}

	pub async fn best_block_hash(&self) -> Result<H256, subxt::Error> {
		self.rpc_chain_get_block_hash(None).await
	}

	pub async fn finalized_block_hash(&self) -> Result<H256, subxt::Error> {
		self.rpc_chain_get_finalized_head().await
	}

	// Block Height
	pub async fn block_height(&self, block_hash: H256) -> Result<u32, subxt::Error> {
		let header = self.rpc_chain_get_header(Some(block_hash)).await?;
		Ok(header.number)
	}

	pub async fn best_block_height(&self) -> Result<u32, subxt::Error> {
		let header = self.rpc_chain_get_header(None).await?;
		Ok(header.number)
	}

	pub async fn finalized_block_height(&self) -> Result<u32, subxt::Error> {
		let block_hash = self.finalized_block_hash().await?;
		let header = self.rpc_chain_get_header(Some(block_hash)).await?;
		Ok(header.number)
	}

	// Nonce
	pub async fn nonce(&self, address: &str) -> Result<u32, subxt::Error> {
		let account = AccountId::from_str(address)?;
		self.rpc_system_account_next_index(account.to_string()).await
	}

	pub async fn nonce_state(&self, address: &str, block_hash: H256) -> Result<u32, subxt::Error> {
		let account = AccountId::from_str(address)?;
		let block = self.online_client.blocks().at(block_hash).await?;

		Ok(block.account_nonce(&account).await? as u32)
	}

	pub async fn best_block_nonce(&self, address: &str) -> Result<u32, subxt::Error> {
		self.nonce_state(address, self.best_block_hash().await?).await
	}

	pub async fn finalized_block_nonce(&self, address: &str) -> Result<u32, subxt::Error> {
		self.nonce_state(address, self.finalized_block_hash().await?).await
	}

	// Balance
	pub async fn balance(&self, account_id: AccountId, at: H256) -> Result<AccountData<u128>, subxt::Error> {
		Ok(self.account_info(account_id, at).await?.data)
	}

	pub async fn best_block_balance(&self, account_id: AccountId) -> Result<AccountData<u128>, subxt::Error> {
		Ok(self.best_block_account_info(account_id).await?.data)
	}

	pub async fn finalized_block_balance(&self, account_id: AccountId) -> Result<AccountData<u128>, subxt::Error> {
		Ok(self.finalized_block_account_info(account_id).await?.data)
	}

	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: AccountId, at: H256) -> Result<Account, subxt::Error> {
		let storage = self.subxt_storage().at(at);
		let address = crate::avail::storage().system().account(account_id);
		storage.fetch_or_default(&address).await
	}

	pub async fn best_block_account_info(&self, account_id: AccountId) -> Result<Account, subxt::Error> {
		let at = self.best_block_hash().await?;
		let storage = self.subxt_storage().at(at);
		let address = crate::avail::storage().system().account(account_id);
		storage.fetch_or_default(&address).await
	}

	pub async fn finalized_block_account_info(&self, account_id: AccountId) -> Result<Account, subxt::Error> {
		let at = self.finalized_block_hash().await?;
		let storage = self.subxt_storage().at(at);
		let address = crate::avail::storage().system().account(account_id);
		storage.fetch_or_default(&address).await
	}

	pub fn subxt_blocks(&self) -> ABlocksClient {
		self.online_client.blocks()
	}

	pub fn subxt_storage(&self) -> AStorageClient {
		self.online_client.storage()
	}

	pub fn subxt_constants(&self) -> AConstantsClient {
		self.online_client.constants()
	}

	pub fn subxt_events(&self) -> AEventsClient {
		AEventsClient::new(self.online_client.clone())
	}

	// Submission
	/// TODO
	pub async fn sign_and_submit<T>(
		&self,
		signer: &Keypair,
		payload: &DefaultPayload<T>,
		options: Options,
	) -> Result<SubmittedTransaction, subxt::Error>
	where
		T: StaticExtrinsic + EncodeAsFields,
	{
		let account_id = signer.public_key().to_account_id();
		let options = options.build(self, &account_id).await?;
		let params = options.clone().build().await;
		if params.6 .0 .0 != 0 && (payload.pallet_name() != "DataAvailability" || payload.call_name() != "submit_data")
		{
			return Err(subxt::Error::Other(
				"Transaction is not compatible with non-zero AppIds".into(),
			));
		}

		let tx_client = self.online_client.tx();
		let signed_call = tx_client.create_signed(payload, signer, params).await?;
		let tx_hash = self.rpc_author_submit_extrinsic(signed_call.encoded()).await?;
		info!(target: "submission", "Transaction submitted. Tx Hash: {:?}, Fork Hash: {:?}, Fork Height: {:?}, Period: {}, Nonce: {}, Account Address: {}", tx_hash, options.mortality.block_hash, options.mortality.block_height, options.mortality.period, options.nonce, account_id);

		Ok(SubmittedTransaction::new(self.clone(), tx_hash, account_id, &options))
	}

	/// TODO
	pub async fn sign_submit_and_watch<T>(
		&self,
		signer: &Keypair,
		payload: &DefaultPayload<T>,
		options: Options,
	) -> Result<TransactionDetails, SubmissionStateError>
	where
		T: StaticExtrinsic + EncodeAsFields,
	{
		let account_id = signer.public_key().to_account_id();
		let info = match self.sign_and_submit(signer, payload, options).await {
			Ok(x) => x,
			Err(err) => return Err(SubmissionStateError::FailedToSubmit { reason: err }),
		};

		let block_id = info.block_id(Default::default()).await;
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
		if let Ok(cache) = self.cache.lock() {
			if let Some(cached_block) = &cache.last_fetched_block {
				if cached_block.0 == block_id.hash {
					block = Some(cached_block.1.clone())
				}
			}
		}

		let block: Arc<ABlock> = if let Some(block) = block {
			block
		} else {
			let block = match self.block_at(block_id.hash).await {
				Ok(x) => x,
				Err(err) => {
					return Err(SubmissionStateError::SubmittedButErrorInSearch {
						tx_hash: info.tx_hash,
						reason: err,
					})
				},
			};
			let block = Arc::new(block);
			if let Ok(mut cache) = self.cache.lock() {
				cache.last_fetched_block = Some((block_id.hash, block.clone()))
			}
			block
		};

		let details = match find_transaction(self, &block, &info.tx_hash).await {
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
				info!(target: "tx_search", "Transaction Found. Tx Hash: {:?}, Tx Index: {}, Block Hash: {:?}, Block Height: {}, Nonce: {}, Account Address: {}", x.tx_hash, x.tx_index, x.block_hash, x.block_number, info.nonce, account_id);
				Ok(x)
			},
			None => Err(SubmissionStateError::Dropped { tx_hash: info.tx_hash }),
		}
	}
}
