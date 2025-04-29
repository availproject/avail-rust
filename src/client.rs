use crate::{
	block::EventRecords,
	error::ClientError,
	rpc::{self, rpc::RpcMethods},
	transaction::{find_transaction, SubmissionStateError, SubmittedTransaction},
	ABlock, ABlocksClient, AConstantsClient, AEventsClient, AOnlineClient, AStorageClient, AvailHeader, Options,
	TransactionDetails,
};
use log::info;
use primitive_types::H256;
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

	pub async fn toggle_tx_state_rpc(&mut self) -> bool {
		let mut value = false;

		// Check if we have tx_state_rpc available to us.
		let methods = self.rpc_methods_list().await.unwrap_or_default();
		if !methods.methods.contains(&String::from("transaction_state")) {
			return value;
		}

		if let Ok(mut lock) = self.options.lock() {
			lock.tx_state_rpc_enabled = !lock.tx_state_rpc_enabled;
			value = lock.tx_state_rpc_enabled;
		}

		value
	}

	pub fn get_options(&self) -> ClientOptions {
		if let Ok(lock) = self.options.lock() {
			return *lock;
		}
		ClientOptions::default()
	}

	pub fn blocks(&self) -> ABlocksClient {
		self.online_client.blocks()
	}

	pub fn storage(&self) -> AStorageClient {
		self.online_client.storage()
	}

	pub fn constants(&self) -> AConstantsClient {
		self.online_client.constants()
	}

	pub fn events(&self) -> AEventsClient {
		AEventsClient::new(self.online_client.clone())
	}

	pub async fn event_records(&self, at: H256) -> Result<Option<EventRecords>, subxt::Error> {
		let events = self.events().at(at).await?;
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

	pub async fn header_at(&self, at: H256) -> Result<AvailHeader, subxt::Error> {
		rpc::chain::get_header(self, Some(at)).await
	}

	pub async fn block_hash(&self, block_height: u32) -> Result<H256, subxt::Error> {
		rpc::chain::get_block_hash(self, Some(block_height)).await
	}

	pub async fn best_block_hash(&self) -> Result<H256, subxt::Error> {
		rpc::chain::get_block_hash(self, None).await
	}

	pub async fn finalized_block_hash(&self) -> Result<H256, subxt::Error> {
		rpc::chain::get_finalized_head(self).await
	}

	pub async fn block_number(&self, block_hash: H256) -> Result<u32, subxt::Error> {
		let header = rpc::chain::get_header(self, Some(block_hash)).await?;
		Ok(header.number)
	}

	pub async fn best_block_number(&self) -> Result<u32, subxt::Error> {
		let header = rpc::chain::get_header(self, None).await?;
		Ok(header.number)
	}

	pub async fn finalized_block_number(&self) -> Result<u32, subxt::Error> {
		let block_hash = self.finalized_block_hash().await?;
		let header = rpc::chain::get_header(self, Some(block_hash)).await?;
		Ok(header.number)
	}

	pub async fn rpc_methods_list(&self) -> Result<RpcMethods, subxt::Error> {
		let methods = rpc::rpc::methods(self).await?;
		Ok(methods)
	}

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
		let tx_hash = rpc::author::submit_extrinsic(self, signed_call.encoded()).await?;

		info!(target: "submission", "Transaction submitted. Tx Hash: {:?}, Fork Hash: {:?}, Fork Height: {:?}, Period: {}, Nonce: {}, Account Address: {}", tx_hash, options.mortality.block_hash, options.mortality.block_number, options.mortality.period, options.nonce, account_id);

		Ok(SubmittedTransaction::new(self.clone(), tx_hash, account_id, options))
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
				info!(target: "tx_search", "Transaction Found. Tx Hash: {:?}, Tx Index: {}, Block Hash: {:?}, Block Height: {}, Nonce: {}, Account Address: {}", x.tx_hash, x.tx_index, x.block_hash, x.block_number, info.nonce(), account_id);
				Ok(x)
			},
			None => Err(SubmissionStateError::Dropped { tx_hash: info.tx_hash }),
		}
	}

	/* 	pub async fn transaction_state(
		&self,
		tx_hash: &H256,
		finalized: bool,
	) -> Result<Vec<TransactionState>, subxt::Error> {
		rpc::transaction::state(self, tx_hash, finalized).await
	} */
}
