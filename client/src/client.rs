//! High-level Avail client combining RPC access with helper APIs for blocks and transactions.

use super::clients::OnlineClient;
use crate::{
	block_api::BlockApi,
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
	/// Connects to an endpoint and returns a ready-to-use client.
	///
	/// Retries automatically if the `reqwest` feature is enabled and `retry` defaults to `true`.
	///
	/// # Errors
	/// Returns `Err(Error)` if the transport cannot be initialized or the handshake fails.
	pub async fn new(endpoint: &str) -> Result<Client, crate::Error> {
		Self::new_ext(endpoint, true).await
	}

	#[cfg(feature = "reqwest")]
	/// Connects to an endpoint; set `retry` to `false` if you prefer failing fast during startup.
	///
	/// Internally wraps [`with_retry_on_error`] so transient errors (network hiccups, etc.) are retried
	/// when `retry` is `true`.
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
	/// # Errors
	/// Propagates any failure returned by [`OnlineClient::new`].
	pub async fn from_rpc_client(rpc_client: RpcClient) -> Result<Client, RpcError> {
		let online_client = OnlineClient::new(&rpc_client).await?;
		Self::from_components(rpc_client, online_client).await
	}

	/// Wraps pre-built components into a handy client handle.
	///
	/// Useful when creating mock transports for testing.
	pub async fn from_components(rpc_client: RpcClient, online_client: OnlineClient) -> Result<Client, RpcError> {
		Ok(Self { online_client, rpc_client })
	}

	#[cfg(feature = "tracing")]
	/// Initializes tracing in plain text or JSON format.
	///
	/// Calling this more than once will return an error from `tracing_subscriber`.
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

	/// Hands back the underlying `OnlineClient` for advanced uses.
	pub fn online_client(&self) -> OnlineClient {
		self.online_client.clone()
	}

	/// Gives you a helper for crafting and sending transactions.
	///
	/// The returned [`TransactionApi`] clones this client and issues RPCs lazily.
	pub fn tx(&self) -> TransactionApi {
		TransactionApi(self.clone())
	}

	/// Builds a block helper rooted at the height or hash you pass in.
	///
	/// See [`BlockApi`] for available views (transactions, events, raw extrinsics).
	pub fn block(&self, block_id: impl Into<HashStringNumber>) -> BlockApi {
		BlockApi::new(self.clone(), block_id)
	}

	/// Provides low-level RPC helpers when you need finer control.
	///
	/// The returned [`Chain`] exposes retry toggles and raw RPC wrappers.
	pub fn chain(&self) -> Chain {
		Chain::new(self.clone())
	}

	/// Provides quick access to the best (head) block view.
	///
	/// Use the returned [`Best`] helper to fetch head metadata or block info repeatedly.
	pub fn best(&self) -> Best {
		Best::new(self.clone())
	}

	/// Provides quick access to finalized block information.
	///
	/// The returned [`Finalized`] helper exposes convenience methods similar to [`Best`].
	pub fn finalized(&self) -> Finalized {
		Finalized::new(self.clone())
	}

	/// Reports whether automatic retries are currently enabled.
	pub fn is_global_retries_enabled(&self) -> bool {
		self.online_client.is_global_retries_enabled()
	}

	/// Turns automatic retries on or off for new requests.
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
