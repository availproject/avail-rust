use jsonrpsee::{
	core::{
		client::{ClientT, Error as JsonClientError},
		traits::ToRpcParams,
	},
	http_client::HttpClient as JsonHttpClient,
};
use serde_json::value::RawValue;
use subxt::backend::rpc::RpcClientT;

pub struct Params(Option<Box<RawValue>>);

impl ToRpcParams for Params {
	fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
		Ok(self.0)
	}
}

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
