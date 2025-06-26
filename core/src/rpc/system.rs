use primitive_types::H256;
use serde::{Deserialize, Serialize};
use subxt_rpcs::{methods::legacy::SystemHealth, rpc_params, RpcClient};

use crate::decoded_transaction::RuntimePhase;

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

pub async fn account_next_index(client: &RpcClient, address: &str) -> Result<u32, subxt_rpcs::Error> {
	let params = rpc_params![address];
	let value = client.request("system_accountNextIndex", params).await?;
	Ok(value)
}

pub async fn chain(client: &RpcClient) -> Result<String, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_chain", params).await?;
	Ok(value)
}

pub async fn chain_type(client: &RpcClient) -> Result<String, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_chainType", params).await?;
	Ok(value)
}

pub async fn health(client: &RpcClient) -> Result<SystemHealth, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_health", params).await?;
	Ok(value)
}

pub async fn local_listen_addresses(client: &RpcClient) -> Result<Vec<String>, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_localListenAddresses", params).await?;
	Ok(value)
}

pub async fn local_peer_id(client: &RpcClient) -> Result<String, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_localPeerId", params).await?;
	Ok(value)
}

pub async fn name(client: &RpcClient) -> Result<String, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_name", params).await?;
	Ok(value)
}

pub async fn node_roles(client: &RpcClient) -> Result<Vec<NodeRole>, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_nodeRoles", params).await?;
	Ok(value)
}

pub async fn peers(client: &RpcClient) -> Result<Vec<PeerInfo>, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_peers", params).await?;
	Ok(value)
}

pub async fn properties(client: &RpcClient) -> Result<SystemProperties, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_properties", params).await?;
	Ok(value)
}

pub async fn sync_state(client: &RpcClient) -> Result<SyncState, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_syncState", params).await?;
	Ok(value)
}

pub async fn version(client: &RpcClient) -> Result<String, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_version", params).await?;
	Ok(value)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchEventsV1Params {
	pub filter: Option<Filter>,
	pub enable_encoding: Option<bool>,
	pub enable_decoding: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum Filter {
	All = 0,
	OnlyExtrinsics = 1,
	OnlyNonExtrinsics = 2,
	Only(Vec<u32>) = 3,
}

impl Default for Filter {
	fn default() -> Self {
		Self::All
	}
}

pub type FetchEventsV1Result = Result<Vec<GroupedRuntimeEvents>, u8>;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct GroupedRuntimeEvents {
	pub phase: RuntimePhase,
	pub events: Vec<RuntimeEvent>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct RuntimeEvent {
	pub index: u32,
	// (Pallet Id, Event Id)
	pub emitted_index: (u8, u8),
	pub encoded: Option<Vec<u8>>,
	pub decoded: Option<Vec<u8>>,
}

pub async fn fetch_events_v1(
	client: &RpcClient,
	params: FetchEventsV1Params,
	at: H256,
) -> Result<FetchEventsV1Result, subxt_rpcs::Error> {
	let params = rpc_params![params, at];
	let value = client.request("system_fetchEventsV1", params).await?;
	Ok(value)
}
