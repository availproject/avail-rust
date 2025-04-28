use crate::{
	block::EventRecords,
	error::ClientError,
	rpc::{self, rpc::RpcMethods},
	ABlock, ABlocksClient, AConstantsClient, AEventsClient, AOnlineClient, AStorageClient, AvailHeader,
	TransactionDetails,
};
use primitive_types::H256;
use std::{fmt::Debug, sync::Arc, time::Duration};
use subxt::backend::rpc::{
	reconnecting_rpc_client::{ExponentialBackoff, RpcClient as ReconnectingRpcClient},
	RpcClient,
};

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
		super::transaction::utils::find_transaction(self, &block, &tx_hash).await
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

	/* 	pub async fn transaction_state(
		&self,
		tx_hash: &H256,
		finalized: bool,
	) -> Result<Vec<TransactionState>, subxt::Error> {
		rpc::transaction::state(self, tx_hash, finalized).await
	} */
}
