use super::Error;
use primitive_types::H256;
use std::str::FromStr;
use subxt_rpcs::{RpcClient, rpc_params};

pub async fn v1_genesishash(client: &RpcClient) -> Result<H256, Error> {
	let value: String = client.request("chainSpec_v1_genesisHash", rpc_params![]).await?;
	let value = H256::from_str(&value).map_err(|e| Error::MalformedResponse(e.to_string()))?;
	Ok(value)
}
