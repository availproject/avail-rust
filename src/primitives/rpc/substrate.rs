use crate::{error::RpcError, primitives::config::TransactionLocation};

use super::AvailHeader;
use primitive_types::H256;
use serde::{Deserialize, Deserializer};
use subxt_core::config::{substrate::BlakeTwo256, Hasher};
use subxt_rpcs::{
	methods::legacy::{RuntimeVersion, SystemHealth},
	rpc_params, RpcClient,
};

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

/// The response from `chain_getBlock`
#[derive(Debug, Clone, Deserialize)]
pub struct SignedBlock {
	/// The block itself.
	pub block: Block,
	/// Block justification.
	pub justifications: Option<Vec<BlockJustification>>,
}

/// Block details in the [`BlockDetails`].
#[derive(Debug, Clone, Deserialize)]
pub struct Block {
	/// The block header.
	pub header: super::AvailHeader,
	#[serde(deserialize_with = "from_string_to_vec")]
	pub extrinsics: Vec<Vec<u8>>,
}

impl Block {
	pub fn has_transaction(&self, tx_hash: H256) -> Option<TransactionLocation> {
		for (i, tx) in self.extrinsics.iter().enumerate() {
			if BlakeTwo256::hash(tx) == tx_hash {
				return Some(TransactionLocation::from((tx_hash, i as u32)));
			}
		}

		None
	}
}

fn from_string_to_vec<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
where
	D: Deserializer<'de>,
{
	let buf = Vec::<String>::deserialize(deserializer)?;
	let result: Result<Vec<Vec<u8>>, _> = buf
		.into_iter()
		.map(|x| hex::decode(x.trim_start_matches("0x")))
		.collect();
	match result {
		Ok(res) => Ok(res),
		Err(err) => Err(serde::de::Error::custom(err)),
	}
}

/// An abstraction over justification for a block's validity under a consensus algorithm.
pub type BlockJustification = (ConsensusEngineId, EncodedJustification);
/// Consensus engine unique ID.
pub type ConsensusEngineId = [u8; 4];
/// The encoded justification specific to a consensus engine.
pub type EncodedJustification = Vec<u8>;

use std::{array::TryFromSliceError, str::FromStr};

#[derive(Debug, Clone)]
pub struct SessionKeys {
	pub babe: [u8; 32],
	pub grandpa: [u8; 32],
	pub im_online: [u8; 32],
	pub authority_discovery: [u8; 32],
}

impl TryFrom<&[u8]> for SessionKeys {
	type Error = String;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		if value.len() != 128 {
			return Err(String::from(
				"Session keys len cannot have length be more or less than 128",
			));
		}

		let err = |e: TryFromSliceError| e.to_string();

		let babe: [u8; 32] = value[0..32].try_into().map_err(err)?;
		let grandpa: [u8; 32] = value[32..64].try_into().map_err(err)?;
		let im_online: [u8; 32] = value[64..96].try_into().map_err(err)?;
		let authority_discovery: [u8; 32] = value[96..128].try_into().map_err(err)?;
		Ok(Self {
			babe,
			grandpa,
			im_online,
			authority_discovery,
		})
	}
}

#[derive(Default, Deserialize)]
pub struct RpcMethods {
	pub methods: Vec<String>,
}

pub async fn system_account_next_index(client: &RpcClient, address: &str) -> Result<u32, subxt_rpcs::Error> {
	let params = rpc_params![address];
	let value = client.request("system_accountNextIndex", params).await?;
	Ok(value)
}

pub async fn system_chain(client: &RpcClient) -> Result<String, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_chain", params).await?;
	Ok(value)
}

pub async fn system_chain_type(client: &RpcClient) -> Result<String, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_chainType", params).await?;
	Ok(value)
}

pub async fn system_health(client: &RpcClient) -> Result<SystemHealth, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_health", params).await?;
	Ok(value)
}

pub async fn system_local_listen_addresses(client: &RpcClient) -> Result<Vec<String>, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_localListenAddresses", params).await?;
	Ok(value)
}

pub async fn system_local_peer_id(client: &RpcClient) -> Result<String, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_localPeerId", params).await?;
	Ok(value)
}

pub async fn system_name(client: &RpcClient) -> Result<String, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_name", params).await?;
	Ok(value)
}

pub async fn system_node_roles(client: &RpcClient) -> Result<Vec<NodeRole>, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_nodeRoles", params).await?;
	Ok(value)
}

pub async fn system_peers(client: &RpcClient) -> Result<Vec<PeerInfo>, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_peers", params).await?;
	Ok(value)
}

pub async fn system_properties(client: &RpcClient) -> Result<SystemProperties, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_properties", params).await?;
	Ok(value)
}

pub async fn system_sync_state(client: &RpcClient) -> Result<SyncState, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_syncState", params).await?;
	Ok(value)
}

pub async fn system_version(client: &RpcClient) -> Result<String, subxt_rpcs::Error> {
	let params = rpc_params![];
	let value = client.request("system_version", params).await?;
	Ok(value)
}

pub async fn chain_get_block(client: &RpcClient, at: Option<H256>) -> Result<Option<SignedBlock>, subxt_rpcs::Error> {
	let params = rpc_params![at];
	let res: Option<SignedBlock> = client.request("chain_getBlock", params).await?;
	let Some(value) = res else { return Ok(None) };
	Ok(Some(value))
}

pub async fn chain_get_block_hash(
	client: &RpcClient,
	block_height: Option<u32>,
) -> Result<Option<H256>, subxt_rpcs::Error> {
	let params = rpc_params![block_height];
	let value = client.request("chain_getBlockHash", params).await?;
	Ok(value)
}

pub async fn chain_get_header(client: &RpcClient, at: Option<H256>) -> Result<Option<AvailHeader>, subxt_rpcs::Error> {
	let params = rpc_params![at];
	let value = client.request("chain_getHeader", params).await?;
	Ok(value)
}

pub async fn author_rotate_keys(client: &RpcClient) -> Result<SessionKeys, RpcError> {
	let params = rpc_params![];
	let value: Vec<u8> = client.request("author_rotateKeys", params).await?;
	let keys = SessionKeys::try_from(value.as_slice())?;
	Ok(keys)
}

pub async fn author_submit_extrinsic(client: &RpcClient, extrinsic: &[u8]) -> Result<H256, subxt_rpcs::Error> {
	let ext = std::format!("0x{}", hex::encode(extrinsic));
	let params = rpc_params![ext];
	let value: H256 = client.request("author_submitExtrinsic", params).await?;
	Ok(value)
}

pub async fn state_call(
	client: &RpcClient,
	method: &str,
	data: &[u8],
	at: Option<H256>,
) -> Result<String, subxt_rpcs::Error> {
	let data = std::format!("0x{}", hex::encode(data));
	let params = rpc_params![method, data, at];
	let value = client.request("state_call", params).await?;
	Ok(value)
}

pub async fn state_get_storage(client: &RpcClient, key: &str, at: Option<H256>) -> Result<Option<Vec<u8>>, RpcError> {
	let params = rpc_params![key, at];
	let value: Option<String> = client.request("state_getStorage", params).await?;
	let Some(value) = value else { return Ok(None) };
	let value = hex::decode(value.trim_start_matches("0x"));
	let value = value.map_err(|e| RpcError::from(e.to_string()))?;
	Ok(Some(value))
}

pub async fn rpc_methods(client: &RpcClient) -> Result<RpcMethods, subxt_rpcs::Error> {
	let value = client.request("rpc_methods", rpc_params![]).await?;
	Ok(value)
}

pub async fn chainspec_v1_genesishash(client: &RpcClient) -> Result<H256, RpcError> {
	let value: String = client.request("chainSpec_v1_genesisHash", rpc_params![]).await?;
	Ok(H256::from_str(&value).map_err(|e| e.to_string())?)
}

pub async fn state_get_metadata(client: &RpcClient, at: Option<H256>) -> Result<Vec<u8>, RpcError> {
	let value: String = client.request("state_getMetadata", rpc_params![at]).await?;
	Ok(hex::decode(value.trim_start_matches("0x")).map_err(|e| e.to_string())?)
}

pub async fn chain_get_finalized_head(client: &RpcClient) -> Result<H256, subxt_rpcs::Error> {
	let value = client.request("chain_getFinalizedHead", rpc_params![]).await?;
	Ok(value)
}

pub async fn state_get_runtime_version(
	client: &RpcClient,
	at: Option<H256>,
) -> Result<RuntimeVersion, subxt_rpcs::Error> {
	let value = client.request("state_getRuntimeVersion", rpc_params![at]).await?;
	Ok(value)
}
