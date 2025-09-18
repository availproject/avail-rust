use crate::{
	platform::spawn,
	subxt_rpcs::{self, RpcClientT, UserError},
};
use serde::Serialize;
use serde_json::value::{RawValue, to_raw_value};
use std::{
	borrow::Cow,
	sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{Receiver, Sender};

/// Serializable [JSON-RPC object](https://www.jsonrpc.org/specification#request-object).
#[derive(Serialize, Debug, Clone)]
pub struct RequestSer<'a> {
	/// JSON-RPC version.
	pub jsonrpc: Cow<'a, str>,
	/// Request ID
	pub id: u64,
	/// Name of the method to be invoked.
	// NOTE: as this type only implements serialize `#[serde(borrow)]` is not needed.
	pub method: Cow<'a, str>,
	/// Parameter values of the request.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub params: Option<Cow<'a, RawValue>>,
}

impl RequestSer<'_> {
	/// Create a owned serializable JSON-RPC method call.
	pub fn owned(id: u64, method: impl Into<String>, params: Option<Box<RawValue>>) -> Self {
		Self {
			jsonrpc: "2.0".into(),
			id,
			method: method.into().into(),
			params: params.map(Cow::Owned),
		}
	}
}

#[derive(Debug, Clone)]
pub struct ResponseError(pub String);

impl std::fmt::Display for ResponseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0.to_string())
	}
}

impl std::error::Error for ResponseError {}

type ResponseMessage = Result<Box<serde_json::Value>, reqwest::Error>;
type ChannelMessage = (Vec<u8>, Sender<ResponseMessage>);

#[derive(Clone)]
pub struct ReqwestClient {
	tx: Sender<ChannelMessage>,
	id: Arc<Mutex<u64>>,
}

impl ReqwestClient {
	pub fn new(endpoint: &str) -> Self {
		let client = Arc::new(reqwest::Client::new());
		let (tx, rx) = tokio::sync::mpsc::channel(1024);
		let endpoint = String::from(endpoint);
		_ = spawn(async move { Self::task(client, endpoint, rx).await });

		let id = Arc::new(Mutex::new(0));
		Self { tx, id }
	}

	async fn task(client: Arc<reqwest::Client>, endpoint: String, mut rx: Receiver<ChannelMessage>) {
		while let Some((body, tx_response)) = rx.recv().await {
			let request = client
				.post(&endpoint)
				.header("Content-Type", "application/json")
				.body(body);

			let response = match request.send().await {
				Ok(x) => x,
				Err(err) => {
					_ = tx_response.send(Err(err)).await;
					continue;
				},
			};

			match response.json::<Box<serde_json::Value>>().await {
				Ok(x) => {
					_ = tx_response.send(Ok(x)).await;
				},
				Err(err) => {
					_ = tx_response.send(Err(err)).await;
				},
			};
		}
	}
}

impl RpcClientT for ReqwestClient {
	fn request_raw<'a>(
		&'a self,
		method: &'a str,
		params: Option<Box<RawValue>>,
	) -> subxt_rpcs::client::RawRpcFuture<'a, Box<RawValue>> {
		Box::pin(async move {
			let request_id = {
				let Ok(mut lock) = self.id.lock() else {
					let err = ResponseError("Failed to acquire lock".into());
					return Err(subxt_rpcs::Error::Client(Box::new(err)));
				};
				let current_id = *lock;
				*lock += 1;
				current_id
			};

			let mut request = {
				let request = RequestSer::owned(request_id, method, params);
				match serde_json::to_vec(&request) {
					Ok(req) => req,
					Err(err) => return Err(subxt_rpcs::Error::Client(Box::new(err))),
				}
			};
			request.shrink_to_fit();

			let (tx, mut rx) = tokio::sync::mpsc::channel(32);
			let message = (request, tx);
			if self.tx.send(message).await.is_err() {
				let err = ResponseError("Failed to send request".into());
				return Err(subxt_rpcs::Error::Client(Box::new(err)));
			}
			let response = match rx.recv().await {
				Some(x) => x,
				None => {
					let err = ResponseError("Failed to receive message".into());
					return Err(subxt_rpcs::Error::Client(Box::new(err)));
				},
			};
			let response = match response {
				Ok(x) => x,
				Err(err) => return Err(subxt_rpcs::Error::Client(Box::new(err))),
			};

			if let Some(Some(response_id)) = response.get("id").map(|x| x.as_u64()) {
				if request_id != response_id {
					let err = ResponseError("Not Pending Request".into());
					return Err(subxt_rpcs::Error::Client(Box::new(err)));
				}
			}
			if let Some(err) = response.get("error") {
				// TODO error message looks like this  "{\"code\":-32601,\"message\":\"Method not found\"}"
				let err = ResponseError(err.to_string());
				return Err(subxt_rpcs::Error::Client(Box::new(err)));
			}
			let Some(result) = response.get("result") else {
				let err = ResponseError("Failed to find result.".into());
				return Err(subxt_rpcs::Error::Client(Box::new(err)));
			};
			match to_raw_value(result) {
				Ok(x) => Ok(x),
				Err(err) => Err(subxt_rpcs::Error::Client(Box::new(err))),
			}
		})
	}

	fn subscribe_raw<'a>(
		&'a self,
		_sub: &'a str,
		_params: Option<Box<RawValue>>,
		_unsub: &'a str,
	) -> subxt_rpcs::client::RawRpcFuture<'a, subxt_rpcs::client::RawRpcSubscription> {
		Box::pin(async move {
			let error = UserError {
				code: 0,
				message: "Subscription call is not implemented".into(),
				data: None,
			};

			Err(subxt_rpcs::Error::User(error))
		})
	}
}

#[cfg(test)]
pub mod testable {
	use std::collections::HashMap;

	use avail_rust_core::{
		AvailHeader, CompactDataLookup, H256, HeaderExtension, KateCommitment, V3HeaderExtension,
		ext::subxt_rpcs::methods::legacy::RuntimeVersion, header::Digest,
	};

	use super::*;

	impl ReqwestClient {
		pub fn new_mocked(return_values: Arc<Mutex<ReturnValues>>) -> Self {
			let id = Arc::new(Mutex::new(0));
			let cloned_id = id.clone();
			let (tx, rx) = tokio::sync::mpsc::channel(1024);
			_ = spawn(async move { Self::task_mocked(return_values, cloned_id, rx).await });

			Self { tx, id }
		}

		async fn task_mocked(
			return_values: Arc<Mutex<ReturnValues>>,
			id: Arc<Mutex<u64>>,
			mut rx: Receiver<ChannelMessage>,
		) {
			while let Some((body, tx_response)) = rx.recv().await {
				/* 				dbg!(&body);
				let response_value = {
					let mut lock = return_values.lock().expect("qed");
					let id = *id.lock().unwrap() - 1;
					let method = body["method"].as_str().unwrap();

					if method == "chain_getFinalizedHead" {
						let v = lock.chain_get_finalized_head.read();
						let ok = OkResponse { id, result: v };
						Box::new(serde_json::to_value(ok).unwrap())
					} else if method == "state_getMetadata" {
						let v = lock.state_get_metadata.read();
						let ok = OkResponse { id, result: v };
						Box::new(serde_json::to_value(ok).unwrap())
					} else if method == "chainSpec_v1_genesisHash" {
						let v = lock.chain_spec_v1_genesis_hash.read();
						let ok = OkResponse { id, result: v };
						Box::new(serde_json::to_value(ok).unwrap())
					} else if method == "state_getRuntimeVersion" {
						let v = lock.state_get_runtime_version.read();
						let ok = OkResponse { id, result: v };
						Box::new(serde_json::to_value(ok).unwrap())
					} else if method == "chain_getHeader" {
						let v = lock.chain_get_header.read();
						let ok = OkResponse { id, result: v };
						Box::new(serde_json::to_value(ok).unwrap())
					} else if method == "chain_getBlockHash" {
						let v = lock.chain_get_block_hash.read();
						let ok = OkResponse { id, result: v };
						Box::new(serde_json::to_value(ok).unwrap())
					} else {
						todo!()
					}
				};
				tx_response.send(Ok(response_value)).await.unwrap(); */
			}
		}
	}

	#[derive(Clone)]
	pub enum ReturnValue<V: Clone + Serialize + Send> {
		RepeatSingleValue(V),
		MultiValues { values: Vec<V> },
	}

	impl<V: Clone + Serialize + Send> ReturnValue<V> {
		pub fn new_single(value: V) -> Self {
			Self::RepeatSingleValue(value)
		}

		pub fn read(&mut self) -> V {
			match self {
				ReturnValue::RepeatSingleValue(x) => x.clone(),
				ReturnValue::MultiValues { values } => values.remove(0),
			}
		}
	}

	#[derive(Clone)]
	pub struct ReturnValues {
		pub chain_get_finalized_head: ReturnValue<H256>,
		pub state_get_metadata: ReturnValue<String>,
		pub chain_spec_v1_genesis_hash: ReturnValue<H256>,
		pub state_get_runtime_version: ReturnValue<RuntimeVersion>,
		pub chain_get_header: ReturnValue<AvailHeader>,
		pub chain_get_block_hash: ReturnValue<H256>,
	}

	impl ReturnValues {
		pub fn new() -> Self {
			let runtime_metadata = include_str!("./test_runtime_metadata.txt").to_string();
			let runtime_version = RuntimeVersion {
				spec_version: 0,
				transaction_version: 0,
				other: HashMap::new(),
			};
			let extension = V3HeaderExtension {
				app_lookup: CompactDataLookup { size: 0, index: Vec::new() },
				commitment: KateCommitment {
					rows: 0,
					cols: 0,
					commitment: Vec::new(),
					data_root: H256::default(),
				},
			};
			let avail_header = AvailHeader {
				parent_hash: H256::default(),
				number: 0,
				state_root: H256::default(),
				extrinsics_root: H256::default(),
				digest: Digest::default(),
				extension: HeaderExtension::V3(extension),
			};
			Self {
				chain_get_finalized_head: ReturnValue::RepeatSingleValue(H256::default()),
				state_get_metadata: ReturnValue::RepeatSingleValue(runtime_metadata),
				chain_spec_v1_genesis_hash: ReturnValue::RepeatSingleValue(H256::default()),
				state_get_runtime_version: ReturnValue::RepeatSingleValue(runtime_version),
				chain_get_header: ReturnValue::RepeatSingleValue(avail_header),
				chain_get_block_hash: ReturnValue::RepeatSingleValue(H256::default()),
			}
		}

		pub fn new_block(&mut self, height: u32, hash: H256) {
			self.chain_get_block_hash = ReturnValue::RepeatSingleValue(hash);
			let mut header = self.chain_get_header.read();
			header.number = height;
			self.chain_get_header = ReturnValue::RepeatSingleValue(header);
		}

		pub fn lock_new_block(this: &Arc<Mutex<Self>>, height: u32, hash: H256) {
			let mut t = this.lock().unwrap();
			t.new_block(height, hash);
		}
	}

	#[derive(Clone, Serialize)]
	struct OkResponse<V: Clone + Serialize + Send> {
		id: u64,
		result: V,
	}
}
