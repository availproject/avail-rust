use primitive_types::H256;
use serde::{Deserialize, Serialize};
use subxt_rpcs::{RpcClient, methods::legacy::SystemHealth, rpc_params};

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

pub async fn fetch_events_v1(
	client: &RpcClient,
	params: fetch_events_v1_types::Params,
	at: H256,
) -> Result<fetch_events_v1_types::Output, subxt_rpcs::Error> {
	let params = rpc_params![params, at];
	let value = client.request("system_fetchEventsV1", params).await?;
	Ok(value)
}

pub async fn fetch_extrinsics_v1(
	client: &RpcClient,
	params: fetch_extrinsics_v1_types::Params,
) -> Result<fetch_extrinsics_v1_types::Output, subxt_rpcs::Error> {
	let params = rpc_params![params];
	let value = client.request("system_fetchExtrinsicsV1", params).await?;
	Ok(value)
}

pub mod fetch_events_v1_types {
	pub use super::*;

	pub type FetchEventsV1Params = Params;
	pub type Output = Vec<GroupedRuntimeEvents>;

	#[derive(Default, Clone, Debug, Serialize, Deserialize)]
	pub struct Params {
		pub filter: Option<Filter>,
		pub enable_encoding: Option<bool>,
		pub enable_decoding: Option<bool>,
	}

	impl Params {
		pub fn new(filter: Option<Filter>, enable_encoding: Option<bool>, enable_decoding: Option<bool>) -> Self {
			Self {
				filter,
				enable_encoding,
				enable_decoding,
			}
		}
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
		pub encoded: Option<String>,
		pub decoded: Option<String>,
	}
}

pub mod fetch_extrinsics_v1_types {
	use super::*;
	use crate::HashIndex;

	pub type Output = Vec<ExtrinsicInformation>;
	pub type FetchExtrinsicsV1Params = Params;

	#[derive(Clone, Serialize, Deserialize)]
	pub struct Params {
		pub block_id: HashIndex,
		pub filter: Option<Filter>,
		pub encode_selector: Option<EncodeSelector>,
	}

	impl Params {
		pub fn new(block_id: HashIndex, filter: Option<Filter>, selector: Option<EncodeSelector>) -> Self {
			Self {
				block_id,
				filter,
				encode_selector: selector,
			}
		}
	}

	#[derive(Default, Clone, Serialize, Deserialize)]
	pub struct Filter {
		pub transaction: Option<TransactionFilter>,
		pub signature: Option<SignatureFilter>,
	}

	impl Filter {
		pub fn new(tx: Option<TransactionFilter>, sig: Option<SignatureFilter>) -> Self {
			Self {
				transaction: tx,
				signature: sig,
			}
		}
	}

	#[derive(Clone, Serialize, Deserialize)]
	#[repr(u8)]
	pub enum EncodeSelector {
		None = 0,
		Call = 1,
		Extrinsic = 2,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct ExtrinsicInformation {
		// Hex string encoded
		pub encoded: Option<String>,
		pub tx_hash: H256,
		pub tx_index: u32,
		pub pallet_id: u8,
		pub call_id: u8,
		pub signature: Option<TransactionSignature>,
	}

	#[derive(Clone, Serialize, Deserialize)]
	pub enum TransactionFilter {
		All,
		TxHash(Vec<H256>),
		TxIndex(Vec<u32>),
		Pallet(Vec<u8>),
		PalletCall(Vec<(u8, u8)>),
	}

	impl TransactionFilter {
		pub fn new() -> Self {
			Self::default()
		}
	}

	impl Default for TransactionFilter {
		fn default() -> Self {
			Self::All
		}
	}

	#[derive(Default, Clone, Serialize, Deserialize)]
	pub struct SignatureFilter {
		pub ss58_address: Option<String>,
		pub app_id: Option<u32>,
		pub nonce: Option<u32>,
	}

	impl SignatureFilter {
		pub fn new(ss58_address: Option<String>, app_id: Option<u32>, nonce: Option<u32>) -> Self {
			Self {
				ss58_address,
				app_id,
				nonce,
			}
		}
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct TransactionSignature {
		pub ss58_address: Option<String>,
		pub nonce: u32,
		pub app_id: u32,
		pub mortality: Option<(u64, u64)>,
	}
}
