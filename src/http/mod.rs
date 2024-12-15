use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::client::Error as JsonClientError;
use jsonrpsee::core::traits::ToRpcParams;
use jsonrpsee::http_client::HttpClient as JsonHttpClient;
use serde_json::value::RawValue;
use subxt::backend::rpc::RpcClientT;
use subxt::error::RpcError;

pub struct Params(Option<Box<RawValue>>);

impl ToRpcParams for Params {
	fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
		Ok(self.0)
	}
}

pub struct HttpClient(pub JsonHttpClient);

impl HttpClient {
	pub fn new(endpoint: &str) -> Self {
		Self(JsonHttpClient::builder().build(endpoint).unwrap())
	}
}

impl RpcClientT for HttpClient {
	fn request_raw<'a>(
		&'a self,
		method: &'a str,
		params: Option<Box<RawValue>>,
	) -> subxt::backend::rpc::RawRpcFuture<'a, Box<RawValue>> {
		Box::pin(async move {
			let res = self.0.request(method, Params(params)).await.unwrap();
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
			let error = Box::new(JsonClientError::HttpNotImplemented);
			Err(RpcError::ClientError(error))
		})
	}
}
