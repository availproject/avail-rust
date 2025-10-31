//! High-level Avail client combining RPC access with helper APIs for blocks and transactions.

use super::clients::OnlineClient;
use crate::{
	block::Block,
	chain::{Best, Chain, Finalized},
	subxt_rpcs::RpcClient,
	transaction_api::TransactionApi,
};
use avail_rust_core::{rpc::Error as RpcError, types::metadata::HashStringNumber};
#[cfg(feature = "tracing")]
use tracing_subscriber::util::TryInitError;

/// Primary entry point used throughout the SDK to interact with the node.
#[derive(Clone)]
pub struct Client {
	online_client: OnlineClient,
	pub rpc_client: RpcClient,
}

impl Client {
	#[cfg(feature = "reqwest")]
	/// Connects to an HTTP endpoint and returns a ready-to-use client.
	///
	/// # Arguments
	/// * `endpoint` - RPC URL (HTTP/S) exposed by an Avail node.
	///
	/// # Returns
	/// Returns a [`Client`] that clones its transports internally.
	///
	/// # Errors
	/// Returns `Err(Error)` if the HTTP transport cannot be initialized or the handshake fails.
	///
	/// # Examples
	/// ```no_run
	/// # use avail_rust_client::Client;
	/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
	/// let client = Client::new("https://turing-rpc.avail.so/rpc").await?;
	/// let best = client.best().block_header().await?;
	/// println!("Best block: {:?}", best.hash());
	/// # Ok(()) }
	/// ```
	pub async fn new(endpoint: &str) -> Result<Client, crate::Error> {
		Self::new_ext(endpoint, true).await
	}

	#[cfg(feature = "reqwest")]
	/// Connects to an endpoint with optional retry behaviour during startup.
	///
	/// # Arguments
	/// * `endpoint` - RPC URL (HTTP/S) exposed by an Avail node.
	/// * `retry` - When `true`, transient connection failures are retried before giving up.
	///
	/// # Returns
	/// Returns a fully initialised [`Client`].
	///
	/// # Errors
	/// Returns `Err(Error)` if the HTTP transport cannot be initialised or metadata bootstrap fails.
	///
	/// # Examples
	/// ```no_run
	/// # use avail_rust_client::Client;
	/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
	/// let client = Client::new_ext("http://127.0.0.1:9944", false).await?;
	/// println!("Retries enabled? {}", client.is_global_retries_enabled());
	/// # Ok(()) }
	/// ```
	pub async fn new_ext(endpoint: &str, retry: bool) -> Result<Client, crate::Error> {
		use super::clients::ReqwestClient;

		let op = async || -> Result<Client, crate::Error> {
			let rpc_client = ReqwestClient::new(endpoint);
			let rpc_client = RpcClient::new(rpc_client);

			Self::from_rpc_client(rpc_client).await.map_err(|e| e.into())
		};

		crate::utils::with_retry_on_error(op, retry).await
	}

	/// Builds a client from an existing RPC transport.
	///
	/// # Arguments
	/// * `rpc_client` - Transport implementing the JSON-RPC client trait.
	///
	/// # Returns
	/// Returns a [`Client`] sharing the provided transport.
	///
	/// # Errors
	/// Propagates any failure returned by [`OnlineClient::new`].
	pub async fn from_rpc_client(rpc_client: RpcClient) -> Result<Client, RpcError> {
		let online_client = OnlineClient::new(&rpc_client).await?;
		Self::from_components(rpc_client, online_client).await
	}

	/// Wraps pre-built components into a handy client handle.
	///
	/// # Arguments
	/// * `rpc_client` - Transport used for RPC calls.
	/// * `online_client` - Metadata cache and retry configuration.
	///
	/// # Returns
	/// Returns a [`Client`] that reuses the provided components.
	pub async fn from_components(rpc_client: RpcClient, online_client: OnlineClient) -> Result<Client, RpcError> {
		Ok(Self { online_client, rpc_client })
	}

	#[cfg(feature = "tracing")]
	/// Initialises tracing in plain text or JSON format.
	///
	/// # Arguments
	/// * `json_format` - When `true`, installs a JSON formatter; otherwise plain text.
	///
	/// # Returns
	/// Returns `Ok(())` once the global subscriber is installed.
	///
	/// # Errors
	/// Returns `Err(TryInitError)` when tracing was already initialised elsewhere.
	///
	/// # Examples
	/// ```no_run
	/// # use avail_rust_client::Client;
	/// Client::init_tracing(false).expect("tracing already initialised");
	/// ```
	pub fn init_tracing(json_format: bool) -> Result<(), TryInitError> {
		use tracing_subscriber::util::SubscriberInitExt;

		let builder = tracing_subscriber::fmt::SubscriberBuilder::default();
		if json_format {
			let builder = builder.json();
			builder.finish().try_init()
		} else {
			builder.finish().try_init()
		}
	}

	/// Hands back the underlying [`OnlineClient`] for advanced uses.
	/// # Returns
	/// Returns a clone of the cached online client state.
	pub fn online_client(&self) -> OnlineClient {
		self.online_client.clone()
	}

	/// Returns a transaction helper for crafting and submitting extrinsics.
	///
	/// # Returns
	/// Returns a [`TransactionApi`] that clones this client and issues RPCs lazily.
	pub fn tx(&self) -> TransactionApi {
		TransactionApi(self.clone())
	}

	/// Builds a block helper rooted at the supplied height or hash.
	///
	/// # Arguments
	/// * `block_id` - Hash, height, or string convertible into a [`HashStringNumber`].
	///
	/// # Returns
	/// Returns a [`Block`] helper exposing extrinsic, event, and metadata views.
	pub fn block(&self, block_id: impl Into<HashStringNumber>) -> Block {
		Block::new(self.clone(), block_id)
	}

	/// Provides low-level RPC helpers when you need finer control.
	///
	/// # Returns
	/// Returns a [`Chain`] handle exposing retry controls and raw RPC wrappers.
	pub fn chain(&self) -> Chain {
		Chain::new(self.clone())
	}

	/// Provides quick access to the best (head) block view.
	///
	/// # Returns
	/// Returns a [`Best`] helper optimised for repeated head queries.
	pub fn best(&self) -> Best {
		Best::new(self.clone())
	}

	/// Provides quick access to finalised block information.
	///
	/// # Returns
	/// Returns a [`Finalized`] helper mirroring [`Best`] convenience methods.
	pub fn finalized(&self) -> Finalized {
		Finalized::new(self.clone())
	}

	/// Reports whether automatic retries are currently enabled.
	///
	/// # Returns
	/// Returns `true` when global retries are on, otherwise `false`.
	pub fn is_global_retries_enabled(&self) -> bool {
		self.online_client.is_global_retries_enabled()
	}

	/// Turns automatic retries on or off for new requests.
	///
	/// # Arguments
	/// * `value` - `true` to enable retries for new helpers, `false` to disable.
	pub fn set_global_retries_enabled(&self, value: bool) {
		self.online_client.set_global_retries_enabled(value);
	}
}

// use crate::{ExtrinsicEvent, ExtrinsicEvents, clients::Client, subxt_core::events::Phase};
// use avail_rust_core::{H256, HashNumber, decoded_events::RawEvent, rpc::system::fetch_events};

// pub const EVENTS_STORAGE_ADDRESS: &str = "0x26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7";

// #[derive(Debug, Clone)]
// pub struct HistoricalEvent {
// 	pub phase: Phase,
// 	// [Pallet_index, Variant_index, Event_data...]
// 	pub bytes: RawEvent,
// 	pub topics: Vec<H256>,
// }

// impl HistoricalEvent {
// 	pub fn emitted_index(&self) -> (u8, u8) {
// 		(self.bytes.pallet_index(), self.bytes.variant_index())
// 	}

// 	pub fn pallet_index(&self) -> u8 {
// 		self.bytes.pallet_index()
// 	}

// 	pub fn variant_index(&self) -> u8 {
// 		self.bytes.variant_index()
// 	}

// 	pub fn event_bytes(&self) -> &[u8] {
// 		&self.bytes.0
// 	}

// 	pub fn event_data(&self) -> &[u8] {
// 		self.bytes.event_data()
// 	}
// }

// #[derive(Clone)]
// pub struct EventClient {
// 	client: Client,
// }

// impl EventClient {
// 	pub fn new(client: Client) -> Self {
// 		Self { client }
// 	}

// 	/// Use this function in case where `transaction_events` or `block_events` do not work.
// 	/// Both mentioned functions require the runtime to have a specific runtime api available which
// 	/// older blocks (runtime) do not have.
// 	pub async fn historical_block_events(&self, at: H256) -> Result<Vec<HistoricalEvent>, RpcError> {
// 		use crate::{config::AvailConfig, subxt_core::events::Events};

// 		let entries = self
// 			.client
// 			.rpc()
// 			.state_get_storage(EVENTS_STORAGE_ADDRESS, Some(at))
// 			.await?;
// 		let Some(event_bytes) = entries else {
// 			return Ok(Vec::new());
// 		};

// 		let mut result: Vec<HistoricalEvent> = Vec::with_capacity(5);
// 		let raw_events = Events::<AvailConfig>::decode_from(event_bytes, self.client.online_client().metadata());
// 		for raw in raw_events.iter() {
// 			let Ok(raw) = raw else {
// 				continue;
// 			};
// 			let mut bytes: Vec<u8> = Vec::with_capacity(raw.field_bytes().len() + 2);
// 			bytes.push(raw.pallet_index());
// 			bytes.push(raw.variant_index());
// 			bytes.append(&mut raw.field_bytes().to_vec());

// 			let Ok(bytes) = RawEvent::try_from(bytes) else {
// 				continue;
// 			};

// 			let value = HistoricalEvent { phase: raw.phase(), bytes, topics: raw.topics().to_vec() };
// 			result.push(value);
// 		}

// 		Ok(result)
// 	}
// }
