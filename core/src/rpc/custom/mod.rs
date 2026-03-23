mod events;
mod extrinsics;

use crate::{rpc::Error, types::metadata::ChainInfo};
use primitive_types::H256;
use subxt_rpcs::{RpcClient, rpc_params};

pub use events::{AllowedEvents, PhaseEvents, RuntimeEvent, fetch_events};
pub use extrinsics::{AllowedExtrinsic, DataFormat, Extrinsic, Query, fetch_extrinsics};

pub async fn get_block_number(client: &RpcClient, at: H256) -> Result<Option<u32>, Error> {
	let params = rpc_params![at];
	let value = client.request("custom_blockNumber", params).await?;
	Ok(value)
}

pub async fn chain_info(client: &RpcClient) -> Result<ChainInfo, Error> {
	let params = rpc_params![];
	let value = client.request("custom_chainInfo", params).await?;
	Ok(value)
}

pub async fn block_timestamp(client: &RpcClient, at: BlockId) -> Result<u64, Error> {
	let params = rpc_params![at];
	let value = client.request("custom_blockTimestamp", params).await?;
	Ok(value)
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum BlockId {
	Hash(H256),
	Number(u32),
}
