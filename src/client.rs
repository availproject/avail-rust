use crate::{
	block::EventRecords,
	error::ClientError,
	rpc::{self, rpc::RpcMethods},
	transaction::BlockId,
	ABlock, ABlocksClient, AConstantsClient, AEventsClient, AOnlineClient, AStorageClient, AvailHeader,
	TransactionState,
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
	let mut client = Client::new(api, rpc_client);
	client.set_mode(ClientMode::HTTP);

	Ok(client)
}

#[derive(Debug, Clone, Copy)]
pub struct ClientOptions {
	pub mode: ClientMode,
	pub tx_state_rpc_enabled: bool,
}

impl Default for ClientOptions {
	fn default() -> Self {
		Self {
			mode: ClientMode::WS,
			tx_state_rpc_enabled: false,
		}
	}
}

type SharedClientOptions = Arc<std::sync::Mutex<ClientOptions>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientMode {
	WS,
	HTTP,
}

#[derive(Debug, Clone)]
pub struct Client {
	pub online_client: AOnlineClient,
	pub rpc_client: RpcClient,
	pub options: SharedClientOptions,
}

impl Client {
	pub fn new(online_client: AOnlineClient, rpc_client: RpcClient) -> Client {
		Self {
			online_client,
			rpc_client,
			options: SharedClientOptions::default(),
		}
	}

	pub fn set_mode(&mut self, value: ClientMode) {
		if let Ok(mut lock) = self.options.lock() {
			lock.mode = value;
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
		self.options.lock().map(|x| *x).unwrap_or_default()
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

	pub async fn header_at(&self, at: H256) -> Result<Option<AvailHeader>, subxt::Error> {
		rpc::chain::get_header(self, Some(at)).await
	}

	pub async fn best_block_header(&self) -> Result<AvailHeader, subxt::Error> {
		let header = rpc::chain::get_header(self, None).await?;
		let Some(header) = header else {
			return Err(subxt::Error::Other("Best block header not found.".into()));
		};
		Ok(header)
	}

	pub async fn finalized_block_header(&self) -> Result<AvailHeader, subxt::Error> {
		let header = self.header_at(self.finalized_block_hash().await?).await?;
		let Some(header) = header else {
			return Err(subxt::Error::Other("Finalized block header not found.".into()));
		};

		Ok(header)
	}

	pub async fn block_hash(&self, block_height: u32) -> Result<Option<H256>, subxt::Error> {
		rpc::chain::get_block_hash(self, Some(block_height)).await
	}

	pub async fn best_block_hash(&self) -> Result<H256, subxt::Error> {
		let hash = rpc::chain::get_block_hash(self, None).await?;
		let Some(hash) = hash else {
			return Err(subxt::Error::Other("Best block hash not found.".into()));
		};

		Ok(hash)
	}

	pub async fn finalized_block_hash(&self) -> Result<H256, subxt::Error> {
		rpc::chain::get_finalized_head(self).await
	}

	pub async fn block_number(&self, block_hash: H256) -> Result<Option<u32>, subxt::Error> {
		let header = rpc::chain::get_header(self, Some(block_hash)).await?;
		Ok(header.map(|x| x.number))
	}

	pub async fn best_block_number(&self) -> Result<u32, subxt::Error> {
		let header = self.best_block_header().await?;
		Ok(header.number)
	}

	pub async fn finalized_block_number(&self) -> Result<u32, subxt::Error> {
		let header = self.finalized_block_header().await?;
		Ok(header.number)
	}

	pub async fn rpc_methods_list(&self) -> Result<RpcMethods, subxt::Error> {
		let methods = rpc::rpc::methods(self).await?;
		Ok(methods)
	}

	// Block Id
	pub async fn best_block_id(&self) -> Result<BlockId, subxt::Error> {
		let hash = self.best_block_hash().await?;
		let height = self.block_number(hash).await?;
		let Some(height) = height else {
			return Err(subxt::Error::Other("Best block header not found.".into()));
		};
		Ok(BlockId::from((hash, height)))
	}

	pub async fn finalized_block_id(&self) -> Result<BlockId, subxt::Error> {
		let hash = self.finalized_block_hash().await?;
		let height = self.block_number(hash).await?;
		let Some(height) = height else {
			return Err(subxt::Error::Other("Finalized block header not found.".into()));
		};
		Ok(BlockId::from((hash, height)))
	}

	pub async fn transaction_state(
		&self,
		tx_hash: &H256,
		finalized: bool,
	) -> Result<Vec<TransactionState>, subxt::Error> {
		rpc::transaction::state(self, tx_hash, finalized).await
	}
}
