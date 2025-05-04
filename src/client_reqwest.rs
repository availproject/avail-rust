use super::platform::spawn;
use serde::Serialize;
use serde_json::value::{to_raw_value, RawValue};
use std::{borrow::Cow, sync::Arc};
use subxt::backend::rpc::RpcClientT;
use tokio::sync::mpsc::{Receiver, Sender};

/// Serializable [JSON-RPC object](https://www.jsonrpc.org/specification#request-object).
#[derive(Serialize, Debug, Clone)]
pub struct RequestSer<'a> {
	/// JSON-RPC version.
	pub jsonrpc: String,
	/// Request ID
	pub id: u32,
	/// Name of the method to be invoked.
	// NOTE: as this type only implements serialize `#[serde(borrow)]` is not needed.
	pub method: Cow<'a, str>,
	/// Parameter values of the request.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub params: Option<Cow<'a, RawValue>>,
}

impl<'a> RequestSer<'a> {
	/// Create a owned serializable JSON-RPC method call.
	pub fn owned(id: u32, method: impl Into<String>, params: Option<Box<RawValue>>) -> Self {
		Self {
			jsonrpc: "2.0".into(),
			id,
			method: method.into().into(),
			params: params.map(Cow::Owned),
		}
	}
}

type ChannelMessage = (serde_json::Value, Sender<Box<serde_json::Value>>);

#[derive(Clone)]
pub struct ReqwestClient {
	endpoint: String,
	tx: Sender<ChannelMessage>,
}

impl ReqwestClient {
	pub fn new(endpoint: &str) -> Self {
		let client = Arc::new(reqwest::Client::new());
		let (tx, rx) = tokio::sync::mpsc::channel(32);
		let end = String::from(endpoint);
		_ = spawn(async move { ReqwestClient::do_work(client, end, rx).await });
		Self {
			endpoint: String::from(endpoint),
			tx,
		}
	}

	pub async fn do_work(client: Arc<reqwest::Client>, endpoint: String, mut rx: Receiver<ChannelMessage>) {
		loop {
			let (body, response) = rx.recv().await.unwrap();
			let a = client
				.post(&endpoint)
				.header("Content-Type", "application/json")
				.json(&body);
			let b = a.send().await.unwrap();
			let c: Box<serde_json::Value> = b.json().await.unwrap();
			response.send(c).await.unwrap();
		}
	}
}

impl RpcClientT for ReqwestClient {
	fn request_raw<'a>(
		&'a self,
		method: &'a str,
		params: Option<Box<RawValue>>,
	) -> subxt::backend::rpc::RawRpcFuture<'a, Box<RawValue>> {
		Box::pin(async move {
			let req = RequestSer::owned(0, method, params);
			let body = serde_json::to_value(&req).unwrap();
			let (tx, mut rx) = tokio::sync::mpsc::channel(32);
			let message = (body, tx);
			self.tx.send(message).await.unwrap();

			let response = rx.recv().await.unwrap();
			let a = &response["result"];

			Ok(to_raw_value(a).unwrap())
		})
	}

	fn subscribe_raw<'a>(
		&'a self,
		_sub: &'a str,
		_params: Option<Box<RawValue>>,
		_unsub: &'a str,
	) -> subxt::backend::rpc::RawRpcFuture<'a, subxt::backend::rpc::RawRpcSubscription> {
		Box::pin(async move {
			todo!();
			/* 			return Err(subxt::ext::subxt_rpcs::Error::Client(Box::new(
				JsonClientError::HttpNotImplemented,
			))); */
		})
	}
}
/*
#[derive(Clone)]
pub struct HttpClient(pub JsonHttpClient);

impl HttpClient {
	pub fn new(endpoint: &str) -> Result<Self, jsonrpsee::core::client::Error> {
		let builder = JsonHttpClient::builder();
		let builder = builder.max_request_size(512 * 1024 * 1024); // 512 MiB
		let builder = builder.max_response_size(512 * 1024 * 1024); // 512 MiB
		Ok(Self(builder.build(endpoint)?))
	}
}

impl RpcClientT for HttpClient {
	fn request_raw<'a>(
		&'a self,
		method: &'a str,
		params: Option<Box<RawValue>>,
	) -> subxt::backend::rpc::RawRpcFuture<'a, Box<RawValue>> {
		Box::pin(async move {
			let res = self.0.request(method, Params(params)).await?;
			Ok(res)
		})
	}

	fn subscribe_raw<'a>(
		&'a self,
		_sub: &'a str,
		_params: Option<Box<RawValue>>,
		_unsub: &'a str,
	) -> subxt::backend::rpc::RawRpcFuture<'a, subxt::backend::rpc::RawRpcSubscription> {
		Box::pin(async move {
			return Err(subxt::ext::subxt_rpcs::Error::Client(Box::new(
				JsonClientError::HttpNotImplemented,
			)));
		})
	}
}
 */
