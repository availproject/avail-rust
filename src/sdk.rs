use crate::{
	error::ClientError, rpc, transactions::Transactions, ABlock, ABlocksClient, AOnlineClient, AStorageClient,
};
use primitive_types::H256;
use std::{fmt::Debug, time::Duration};
use subxt::backend::rpc::{
	reconnecting_rpc_client::{ExponentialBackoff, RpcClient as ReconnectingRpcClient},
	RpcClient,
};

#[cfg(feature = "native")]
use crate::http;

#[derive(Clone)]
pub struct SDK {
	pub client: Client,
	pub tx: Transactions,
}

impl SDK {
	pub async fn new(endpoint: &str) -> Result<Self, ClientError> {
		let client = reconnecting_api(endpoint).await?;

		Self::new_custom(client).await
	}

	pub async fn new_custom(client: Client) -> Result<Self, ClientError> {
		let tx = Transactions::new(client.clone());

		Ok(SDK { client, tx })
	}

	pub fn enable_logging() {
		env_logger::builder().init();
	}

	pub fn one_avail() -> u128 {
		1_000_000_000_000_000_000u128
	}

	pub fn local_endpoint() -> &'static str {
		"ws://127.0.0.1:9944"
	}

	pub fn local_http_endpoint() -> &'static str {
		"http://127.0.0.1:9944"
	}

	pub fn turing_endpoint() -> &'static str {
		"wss://turing-rpc.avail.so/ws"
	}

	pub fn turing_http_endpoint() -> &'static str {
		"https://turing-rpc.avail.so/rpc"
	}

	pub fn mainnet_endpoint() -> &'static str {
		"wss://mainnet-rpc.avail.so/ws"
	}

	pub fn mainnet_http_endpoint() -> &'static str {
		"https://mainnet-rpc.avail.so/rpc"
	}
}

#[cfg(feature = "native")]
impl SDK {
	pub async fn new_http(endpoint: &str) -> Result<Self, ClientError> {
		let client = http_api(endpoint).await?;

		Self::new_custom(client).await
	}
}

impl Debug for SDK {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let genesis_hash = self.client.online_client.genesis_hash();
		f.debug_struct("SDK")
			.field("Genesis Hash", &genesis_hash)
			.finish_non_exhaustive()
	}
}

pub async fn reconnecting_api(endpoint: &str) -> Result<Client, ClientError> {
	let rpc_client = ReconnectingRpcClient::builder()
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaitFor {
	BlockInclusion,
	BlockFinalization,
}

impl WaitFor {
	pub fn to_str(&self) -> &'static str {
		match self {
			WaitFor::BlockInclusion => "Block Inclusion",
			WaitFor::BlockFinalization => "Block Finalization",
		}
	}
}

#[derive(Debug, Clone)]
pub struct Client {
	pub online_client: AOnlineClient,
	pub rpc_client: RpcClient,
	pub mode: ClientMode,
}

impl Client {
	pub fn new(online_client: AOnlineClient, rpc_client: RpcClient) -> Client {
		Self {
			online_client,
			rpc_client,
			mode: ClientMode::WS,
		}
	}

	pub fn set_mode(&mut self, value: ClientMode) {
		self.mode = value;
	}

	pub fn blocks(&self) -> ABlocksClient {
		self.online_client.blocks()
	}

	pub fn storage(&self) -> AStorageClient {
		self.online_client.storage()
	}

	pub async fn block_at(&self, at: H256) -> Result<ABlock, subxt::Error> {
		self.online_client.blocks().at(at).await
	}

	pub async fn best_block_hash(&self) -> Result<H256, subxt::Error> {
		rpc::chain::get_block_hash(self, None).await
	}

	pub async fn finalized_block_hash(&self) -> Result<H256, subxt::Error> {
		rpc::chain::get_finalized_head(self).await
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientMode {
	WS,
	HTTP,
}
