use crate::{
	clients::ReqwestClient,
	ext::subxt_rpcs::{self, RpcClientT},
};
use avail_rust_core::grandpa::GrandpaJustification;
use codec::Encode;
use serde_json::value::RawValue;
use std::sync::{Arc, Mutex};

use avail_rust_core::rpc::system::fetch_extrinsics::ExtrinsicInformation;

/// RPC client wrapper that allows injecting canned responses for testing.
#[derive(Clone)]
pub struct MockClient {
	org: ReqwestClient,
	commander: Arc<Mutex<CommandManager>>,
}

impl MockClient {
	/// Creates a mockable RPC client targeting the provided endpoint.
	///
	/// # Arguments
	/// * `endpoint` - HTTP URL of the JSON-RPC server.
	///
	/// # Returns
	/// Returns the client alongside a helper that queues mocked responses.
	pub fn new(endpoint: &str) -> (Self, CommandManagerHelper) {
		let org = ReqwestClient::new(endpoint);
		let commander = Arc::new(Mutex::new(CommandManager::default()));
		let wrapper = CommandManagerHelper { 0: commander.clone() };
		(Self { org, commander }, wrapper)
	}
}

impl RpcClientT for MockClient {
	fn request_raw<'a>(
		&'a self,
		method: &'a str,
		params: Option<Box<RawValue>>,
	) -> subxt_rpcs::client::RawRpcFuture<'a, Box<RawValue>> {
		{
			let mut commander = self.commander.lock().unwrap();
			if let Some(value) = commander.find(method) {
				//println!("Found Mock value: Method: {}", method);
				return Box::pin(async move { value });
			}
		}

		self.org.request_raw(method, params)
	}

	fn subscribe_raw<'a>(
		&'a self,
		sub: &'a str,
		params: Option<Box<RawValue>>,
		unsub: &'a str,
	) -> subxt_rpcs::client::RawRpcFuture<'a, subxt_rpcs::client::RawRpcSubscription> {
		self.org.subscribe_raw(sub, params, unsub)
	}
}

/// Stores queued responses to satisfy upcoming mock RPC calls.
#[derive(Debug, Default)]
pub struct CommandManager {
	list: Vec<(String, Result<Box<RawValue>, subxt_rpcs::Error>)>,
}

impl CommandManager {
	/// Retrieves and removes the earliest queued response for the given method.
	///
	/// # Arguments
	/// * `method` - RPC method name expected to receive the response.
	///
	/// # Returns
	/// Returns the stored response result, or `None` if nothing was queued.
	pub fn find(&mut self, method: &str) -> Option<Result<Box<RawValue>, subxt_rpcs::Error>> {
		let pos = self.list.iter().position(|x| x.0.as_str() == method);
		let Some(pos) = pos else {
			return None;
		};
		let value = self.list.remove(pos);
		Some(value.1)
	}

	/// Queues a successful response for the given method.
	///
	/// # Arguments
	/// * `method` - RPC method name the response targets.
	/// * `value` - Serialized JSON-RPC result payload.
	///
	/// # Returns
	/// Returns `()` once the response is stored.
	pub fn add_ok(&mut self, method: impl Into<String>, value: Box<RawValue>) {
		self.list.push((method.into(), Ok(value)));
	}

	/// Queues an error response for the given method.
	///
	/// # Arguments
	/// * `method` - RPC method name the error targets.
	/// * `value` - Error returned to the caller when dequeued.
	///
	/// # Returns
	/// Returns `()` once the error is stored.
	pub fn add_err(&mut self, method: impl Into<String>, value: subxt_rpcs::Error) {
		self.list.push((method.into(), Err(value)));
	}
}

/// Thread-safe helper that exposes ergonomic methods to queue mock responses.
pub struct CommandManagerHelper(pub Arc<Mutex<CommandManager>>);
impl CommandManagerHelper {
	/// Queues a successful response for the given method.
	///
	/// # Arguments
	/// * `method` - RPC method name targeted by the response.
	/// * `value` - Serialized JSON payload to return.
	///
	/// # Returns
	/// Returns `()` once the response is queued.
	pub fn add_ok(&mut self, method: impl Into<String>, value: Box<RawValue>) {
		let mut lock = self.0.lock().unwrap();
		lock.add_ok(method, value);
	}

	/// Queues an error response for the given method.
	///
	/// # Arguments
	/// * `method` - RPC method name targeted by the error.
	/// * `value` - Error returned when the entry is dequeued.
	///
	/// # Returns
	/// Returns `()` once the error is queued.
	pub fn add_err(&mut self, method: impl Into<String>, value: subxt_rpcs::Error) {
		let mut lock = self.0.lock().unwrap();
		lock.add_err(method, value);
	}

	/// Queues a `grandpa_blockJustification` response containing encoded justification bytes.
	///
	/// # Arguments
	/// * `value` - Optional justification to encode into the response.
	///
	/// # Returns
	/// Returns `()` once the response is queued.
	pub fn justification_ok(&mut self, value: Option<GrandpaJustification>) {
		let value = match value.clone() {
			Some(x) => {
				let value = serde_json::to_string(&Some(const_hex::encode(x.encode()))).unwrap();
				RawValue::from_string(value).unwrap()
			},
			None => {
				let value = serde_json::to_string(&value).unwrap();
				RawValue::from_string(value).unwrap()
			},
		};
		self.add_ok("grandpa_blockJustification", value);
	}

	/// Queues an error response for `grandpa_blockJustification`.
	///
	/// # Arguments
	/// * `value` - Optional error to enqueue; defaults to a reconnect error.
	///
	/// # Returns
	/// Returns `()` once the error response is queued.
	pub fn justification_err(&mut self, value: Option<subxt_rpcs::Error>) {
		let value = value.unwrap_or_else(|| subxt_rpcs::Error::DisconnectedWillReconnect("Error".into()));
		self.add_err("grandpa_blockJustification", value);
	}

	/// Queues a `grandpa_blockJustificationJson` response with raw justification bytes.
	///
	/// # Arguments
	/// * `value` - Optional justification serialized directly into JSON.
	///
	/// # Returns
	/// Returns `()` once the response is queued.
	pub fn justification_json_ok(&mut self, value: Option<GrandpaJustification>) {
		let value = match value.clone() {
			Some(x) => {
				let value = serde_json::to_string(&Some(x)).unwrap();
				RawValue::from_string(value).unwrap()
			},
			None => {
				let value = serde_json::to_string(&value).unwrap();
				RawValue::from_string(value).unwrap()
			},
		};
		self.add_ok("grandpa_blockJustificationJson", value);
	}

	/// Queues an error response for `grandpa_blockJustificationJson`.
	///
	/// # Arguments
	/// * `value` - Optional error to enqueue; defaults to a reconnect error.
	///
	/// # Returns
	/// Returns `()` once the error response is queued.
	pub fn justification_json_err(&mut self, value: Option<subxt_rpcs::Error>) {
		let value = value.unwrap_or_else(|| subxt_rpcs::Error::DisconnectedWillReconnect("Error".into()));
		self.add_err("grandpa_blockJustificationJson", value);
	}

	/// Queues a successful `system_fetchExtrinsicsV1` response.
	///
	/// # Arguments
	/// * `value` - Extrinsic information returned by the node.
	///
	/// # Returns
	/// Returns `()` once the response is queued.
	pub fn extrinsics_ok(&mut self, value: Vec<ExtrinsicInformation>) {
		let value = serde_json::to_string(&value).unwrap();
		let value = RawValue::from_string(value).unwrap();
		self.add_ok("system_fetchExtrinsicsV1", value);
	}

	/// Queues an error response for `system_fetchExtrinsicsV1`.
	///
	/// # Arguments
	/// * `value` - Optional error to enqueue; defaults to a reconnect error.
	///
	/// # Returns
	/// Returns `()` once the error response is queued.
	pub fn extrinsics_err(&mut self, value: Option<subxt_rpcs::Error>) {
		let value = value.unwrap_or_else(|| subxt_rpcs::Error::DisconnectedWillReconnect("Error".into()));
		self.add_err("system_fetchExtrinsicsV1", value);
	}

	/// Queues an error response for `chain_getBlock`.
	///
	/// # Arguments
	/// * `value` - Optional error to enqueue; defaults to a reconnect error.
	///
	/// # Returns
	/// Returns `()` once the error response is queued.
	pub fn legacy_block_err(&mut self, value: Option<subxt_rpcs::Error>) {
		let value = value.unwrap_or_else(|| subxt_rpcs::Error::DisconnectedWillReconnect("Error".into()));
		self.add_err("chain_getBlock", value);
	}

	/// Queues an error response for `chain_getHeader`.
	///
	/// # Arguments
	/// * `value` - Optional error to enqueue; defaults to a reconnect error.
	///
	/// # Returns
	/// Returns `()` once the error response is queued.
	pub fn block_header_err(&mut self, value: Option<subxt_rpcs::Error>) {
		let value = value.unwrap_or_else(|| subxt_rpcs::Error::DisconnectedWillReconnect("Error".into()));
		self.add_err("chain_getHeader", value);
	}
}
