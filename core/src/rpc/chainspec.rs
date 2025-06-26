use crate::error::Error;
use primitive_types::H256;
use std::str::FromStr;
use subxt_rpcs::{rpc_params, RpcClient};

pub async fn v1_genesishash(client: &RpcClient) -> Result<H256, Error> {
	let value: String = client.request("chainSpec_v1_genesisHash", rpc_params![]).await?;
	Ok(H256::from_str(&value).map_err(|e| e.to_string())?)
}
