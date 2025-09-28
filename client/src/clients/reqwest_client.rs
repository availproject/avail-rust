use crate::{
	platform::{AsyncOp, AsyncReceiver, AsyncSender, async_channel, async_recv},
	subxt_rpcs::{self, RpcClientT, UserError},
};
use serde::Serialize;
use serde_json::value::{RawValue, to_raw_value};
use std::{
	borrow::Cow,
	sync::{Arc, Mutex},
};

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
type ChannelMessage = (Vec<u8>, AsyncSender<ResponseMessage>);

#[derive(Clone)]
pub struct ReqwestClient {
	tx: AsyncSender<ChannelMessage>,
	id: Arc<Mutex<u64>>,
}

impl ReqwestClient {
	pub fn new(endpoint: &str, async_op: Arc<dyn AsyncOp>) -> Self {
		let client = Arc::new(reqwest::Client::new());
		let (tx, rx) = async_channel(1024);
		let endpoint = String::from(endpoint);
		let task = async move { Self::task(client, endpoint, rx).await };
		async_op.spawn(Box::pin(task));

		let id = Arc::new(Mutex::new(0));
		Self { tx, id }
	}

	async fn task(client: Arc<reqwest::Client>, endpoint: String, mut rx: AsyncReceiver<ChannelMessage>) {
		while let Some((body, tx_response)) = async_recv(&mut rx).await {
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

			let response = match response.error_for_status() {
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

			let (tx, mut rx) = async_channel(32);
			let message = (request, tx);
			if self.tx.send(message).await.is_err() {
				let err = ResponseError("Failed to send request".into());
				return Err(subxt_rpcs::Error::Client(Box::new(err)));
			}
			let response = match async_recv(&mut rx).await {
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
