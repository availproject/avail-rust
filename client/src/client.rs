//! High-level Avail client combining RPC access with helper APIs for blocks and transactions.

use super::clients::OnlineClient;
use crate::{
	block::Block,
	chain::{Best, Chain, Finalized, Head, HeadKind},
	retry_policy::RetryPolicy,
	subscription::SubscribeApi,
	subxt_rpcs::RpcClient,
	transaction_api::TransactionApi,
};
use avail_rust_core::{rpc::Error as RpcError, types::metadata::HashStringNumber};
#[cfg(feature = "tracing")]
use tracing_subscriber::util::TryInitError;

/// Output format used by [`Client::init_tracing`].
#[cfg(feature = "tracing")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TracingFormat {
	/// Human-friendly text logs.
	#[default]
	Plain,
	/// Structured JSON logs.
	Json,
}

/// Primary entry point used throughout the SDK to interact with the node.
#[derive(Clone)]
pub struct Client {
	online_client: OnlineClient,
	pub rpc_client: RpcClient,
}

impl std::fmt::Debug for Client {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Client").finish()
	}
}

/// Controls how a [`Client`] connects to an RPC endpoint.
#[cfg(feature = "reqwest")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionOptions {
	pub retry_policy: RetryPolicy,
}

#[cfg(feature = "reqwest")]
impl Default for ConnectionOptions {
	fn default() -> Self {
		Self { retry_policy: RetryPolicy::Enabled }
	}
}

impl Client {
	#[cfg(feature = "reqwest")]
	/// Connects to an HTTP endpoint.
	/// Returns an error if transport initialization or handshake fails.
	///
	/// # Examples
	/// ```no_run
	/// # use avail_rust_client::Client;
	/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
	/// let client = Client::connect("https://turing-rpc.avail.so/rpc").await?;
	/// let best = client.best().block_header().await?;
	/// println!("Best block: {:?}", best.hash());
	/// # Ok(()) }
	/// ```
	pub async fn connect(endpoint: &str) -> Result<Client, crate::Error> {
		Self::connect_with(endpoint, ConnectionOptions::default()).await
	}

	#[cfg(feature = "reqwest")]
	/// Connects to an HTTP endpoint with custom connection options.
	/// Returns an error if transport initialization or metadata bootstrap fails.
	///
	/// # Examples
	/// ```no_run
	/// # use avail_rust_client::Client;
	/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
	/// let client = Client::connect_with(
	///     "http://127.0.0.1:9944",
	///     ConnectionOptions { retry_policy: RetryPolicy::Disabled },
	/// ).await?;
	/// println!("Retries enabled? {}", client.retry_policy() != RetryPolicy::Disabled);
	/// # Ok(()) }
	/// ```
	pub async fn connect_with(endpoint: &str, options: ConnectionOptions) -> Result<Client, crate::Error> {
		use super::clients::ReqwestClient;

		retry!(options.retry_policy.resolve(true), {
			let rpc_client = ReqwestClient::new(endpoint);
			let rpc_client = RpcClient::new(rpc_client);
			Self::from_rpc_client(rpc_client).await.map_err(|e| e.into())
		})
		.map(|client| {
			client.set_retry_policy(options.retry_policy);
			client
		})
	}

	/// Builds a client from an existing RPC transport.
	/// Returns an error if metadata/bootstrap queries fail.
	pub async fn from_rpc_client(rpc_client: RpcClient) -> Result<Client, RpcError> {
		let online_client = OnlineClient::new(&rpc_client).await?;
		Self::from_components(rpc_client, online_client).await
	}

	/// Wraps pre-built transport and online metadata state into a client.
	pub async fn from_components(rpc_client: RpcClient, online_client: OnlineClient) -> Result<Client, RpcError> {
		Ok(Self { online_client, rpc_client })
	}

	#[cfg(feature = "tracing")]
	/// Initializes tracing in the selected output format.
	/// Returns an error if a global tracing subscriber is already installed.
	///
	/// # Examples
	/// ```no_run
	/// # use avail_rust_client::Client;
	/// Client::init_tracing(TracingFormat::Plain).expect("tracing already initialized");
	/// ```
	pub fn init_tracing(format: TracingFormat) -> Result<(), TryInitError> {
		use tracing_subscriber::util::SubscriberInitExt;

		let builder = tracing_subscriber::fmt::SubscriberBuilder::default();
		if matches!(format, TracingFormat::Json) {
			let builder = builder.json();
			builder.finish().try_init()
		} else {
			builder.finish().try_init()
		}
	}

	/// Returns the underlying [`OnlineClient`] clone.
	pub fn online_client(&self) -> OnlineClient {
		self.online_client.clone()
	}

	/// Returns a transaction API handle.
	pub fn tx(&self) -> TransactionApi {
		TransactionApi(self.clone())
	}

	/// Returns a block handle for a specific hash or height.
	pub fn block(&self, at: impl Into<HashStringNumber>) -> Block {
		Block::new(self.clone(), at)
	}

	/// Returns low-level chain RPC helpers.
	pub fn chain(&self) -> Chain {
		Chain::new(self.clone())
	}

	/// Returns convenience helpers for the best head.
	pub fn best(&self) -> Best {
		Best::new(self.clone())
	}

	/// Returns convenience helpers for the finalized head.
	pub fn finalized(&self) -> Finalized {
		Finalized::new(self.clone())
	}

	/// Returns head helpers for a selected head kind.
	pub fn head(&self, kind: HeadKind) -> Head {
		Head::new(self.clone(), kind)
	}

	/// Returns the client's current retry policy.
	pub fn retry_policy(&self) -> RetryPolicy {
		self.online_client.retry_policy()
	}

	/// Sets the client-wide retry policy.
	pub fn set_retry_policy(&self, value: RetryPolicy) {
		self.online_client.set_retry_policy(value);
	}

	pub fn subscribe(&self) -> SubscribeApi {
		SubscribeApi(self.clone())
	}

	pub fn account<'a>(&'a self) -> crate::account::Account<'a> {
		crate::account::Account::new(self)
	}
}
