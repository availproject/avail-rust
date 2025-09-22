pub mod fetch_events;
pub mod fetch_extrinsics;

use crate::{BlockRef, rpc::Error};
use primitive_types::H256;
use serde::Deserialize;
use subxt_rpcs::{RpcClient, methods::legacy::SystemHealth, rpc_params};

pub use fetch_events::fetch_events_v1;
pub use fetch_extrinsics::fetch_extrinsics_v1;

/// Network Peer information
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
	/// Peer ID
	pub peer_id: String,
	/// Roles
	pub roles: String,
	/// Peer best block hash
	pub best_hash: H256,
	/// Peer best block number
	pub best_number: u32,
}

/// Arbitrary properties defined in chain spec as a JSON object
pub type SystemProperties = serde_json::map::Map<String, serde_json::Value>;

/// The role the node is running as
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum NodeRole {
	/// The node is a full node
	Full,
	/// The node is an authority
	Authority,
}

/// The state of the syncing of the node.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
	/// Height of the block at which syncing started.
	pub starting_block: u32,
	/// Height of the current best block of the node.
	pub current_block: u32,
	/// Height of the highest block in the network.
	pub highest_block: u32,
}

pub async fn account_next_index(client: &RpcClient, address: &str) -> Result<u32, Error> {
	let params = rpc_params![address];
	let value = client.request("system_accountNextIndex", params).await?;
	Ok(value)
}

pub async fn chain(client: &RpcClient) -> Result<String, Error> {
	let params = rpc_params![];
	let value = client.request("system_chain", params).await?;
	Ok(value)
}

pub async fn chain_type(client: &RpcClient) -> Result<String, Error> {
	let params = rpc_params![];
	let value = client.request("system_chainType", params).await?;
	Ok(value)
}

pub async fn health(client: &RpcClient) -> Result<SystemHealth, Error> {
	let params = rpc_params![];
	let value = client.request("system_health", params).await?;
	Ok(value)
}

pub async fn local_listen_addresses(client: &RpcClient) -> Result<Vec<String>, Error> {
	let params = rpc_params![];
	let value = client.request("system_localListenAddresses", params).await?;
	Ok(value)
}

pub async fn local_peer_id(client: &RpcClient) -> Result<String, Error> {
	let params = rpc_params![];
	let value = client.request("system_localPeerId", params).await?;
	Ok(value)
}

pub async fn name(client: &RpcClient) -> Result<String, Error> {
	let params = rpc_params![];
	let value = client.request("system_name", params).await?;
	Ok(value)
}

pub async fn node_roles(client: &RpcClient) -> Result<Vec<NodeRole>, Error> {
	let params = rpc_params![];
	let value = client.request("system_nodeRoles", params).await?;
	Ok(value)
}

pub async fn peers(client: &RpcClient) -> Result<Vec<PeerInfo>, Error> {
	let params = rpc_params![];
	let value = client.request("system_peers", params).await?;
	Ok(value)
}

pub async fn properties(client: &RpcClient) -> Result<SystemProperties, Error> {
	let params = rpc_params![];
	let value = client.request("system_properties", params).await?;
	Ok(value)
}

pub async fn sync_state(client: &RpcClient) -> Result<SyncState, Error> {
	let params = rpc_params![];
	let value = client.request("system_syncState", params).await?;
	Ok(value)
}

pub async fn version(client: &RpcClient) -> Result<String, Error> {
	let params = rpc_params![];
	let value = client.request("system_version", params).await?;
	Ok(value)
}

pub async fn get_block_number(client: &RpcClient, at: H256) -> Result<Option<u32>, Error> {
	let params = rpc_params![at];
	let value = client.request("system_getBlockNumber", params).await?;
	Ok(value)
}

pub async fn latest_block_info(client: &RpcClient, use_best_block: bool) -> Result<BlockRef, Error> {
	let params = rpc_params![use_best_block];
	let value = client.request("system_latestBlockInfo", params).await?;
	Ok(value)
}
