use crate::error::Error;
use primitive_types::H256;
use subxt_rpcs::{RpcClient, methods::legacy::RuntimeVersion, rpc_params};

pub async fn call(
	client: &RpcClient,
	method: &str,
	data: &[u8],
	at: Option<H256>,
) -> Result<String, subxt_rpcs::Error> {
	let data = std::format!("0x{}", const_hex::encode(data));
	let params = rpc_params![method, data, at];
	let value = client.request("state_call", params).await?;
	Ok(value)
}

pub async fn get_storage(client: &RpcClient, key: &str, at: Option<H256>) -> Result<Option<Vec<u8>>, Error> {
	let params = rpc_params![key, at];
	let value: Option<String> = client.request("state_getStorage", params).await?;
	let Some(value) = value else {
		return Ok(None);
	};
	let value = const_hex::decode(value.trim_start_matches("0x"));
	let value = value.map_err(|e| Error::from(e.to_string()))?;
	Ok(Some(value))
}

pub async fn get_keys_paged(
	client: &RpcClient,
	prefix: Option<String>,
	count: u32,
	start_key: Option<String>,
	at: Option<H256>,
) -> Result<Vec<String>, Error> {
	let params = rpc_params![prefix, count, start_key, at];
	let value: Vec<String> = client.request("state_getKeysPaged", params).await?;
	Ok(value)
}

pub async fn get_metadata(client: &RpcClient, at: Option<H256>) -> Result<Vec<u8>, Error> {
	let value: String = client.request("state_getMetadata", rpc_params![at]).await?;
	Ok(const_hex::decode(value.trim_start_matches("0x")).map_err(|e| e.to_string())?)
}

pub async fn get_runtime_version(client: &RpcClient, at: Option<H256>) -> Result<RuntimeVersion, subxt_rpcs::Error> {
	let value = client.request("state_getRuntimeVersion", rpc_params![at]).await?;
	Ok(value)
}
