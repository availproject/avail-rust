use serde::Deserialize;
use subxt_rpcs::{RpcClient, rpc_params};

#[derive(Default, Deserialize)]
pub struct RpcMethods {
	pub methods: Vec<String>,
}

pub async fn call(client: &RpcClient) -> Result<RpcMethods, subxt_rpcs::Error> {
	let value = client.request("rpc_methods", rpc_params![]).await?;
	Ok(value)
}
