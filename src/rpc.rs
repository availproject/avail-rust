use crate::{
	avail::runtime_types::{da_runtime::primitives::SessionKeys, frame_system::limits::BlockLength},
	from_substrate::{NodeRole, PeerInfo, SyncState},
	utils, ABlockDetailsRPC, AvailHeader, BlockNumber, Cell, Client, GDataProof, GRow, H256,
};
use avail_core::data_proof::ProofResponse;
use serde::{Deserialize, Serialize};
use subxt::{
	backend::legacy::rpc_methods::{Bytes, RuntimeVersion, SystemHealth},
	rpc_params,
};

/// Arbitrary properties defined in chain spec as a JSON object
pub type SystemProperties = serde_json::map::Map<String, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionState {
	pub block_hash: H256,
	pub block_height: u32,
	pub tx_hash: H256,
	pub tx_index: u32,
	pub tx_success: bool,
	pub pallet_index: u8,
	pub call_index: u8,
	pub is_finalized: bool,
}

pub mod transaction {
	use super::*;
	pub async fn state(
		client: &Client,
		tx_hash: &H256,
		finalized: bool,
	) -> Result<Vec<TransactionState>, subxt::Error> {
		let params = rpc_params![tx_hash, finalized];
		let value = client.rpc_client.request("transaction_state", params).await?;
		Ok(value)
	}
}

pub mod system {
	use super::*;

	pub async fn account_next_index(client: &Client, account: String) -> Result<u32, subxt::Error> {
		let params = rpc_params![account];
		let value = client.rpc_client.request("system_accountNextIndex", params).await?;
		Ok(value)
	}

	pub async fn chain(client: &Client) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_chain", params).await?;
		Ok(value)
	}

	pub async fn chain_type(client: &Client) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_chainType", params).await?;
		Ok(value)
	}

	pub async fn health(client: &Client) -> Result<SystemHealth, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_health", params).await?;
		Ok(value)
	}

	pub async fn local_listen_addresses(client: &Client) -> Result<Vec<String>, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_localListenAddresses", params).await?;
		Ok(value)
	}

	pub async fn local_peer_id(client: &Client) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_localPeerId", params).await?;
		Ok(value)
	}

	pub async fn name(client: &Client) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_name", params).await?;
		Ok(value)
	}

	pub async fn node_roles(client: &Client) -> Result<Vec<NodeRole>, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_nodeRoles", params).await?;
		Ok(value)
	}

	pub async fn peers(client: &Client) -> Result<Vec<PeerInfo>, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_peers", params).await?;
		Ok(value)
	}

	pub async fn properties(client: &Client) -> Result<SystemProperties, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_properties", params).await?;
		Ok(value)
	}

	pub async fn sync_state(client: &Client) -> Result<SyncState, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_syncState", params).await?;
		Ok(value)
	}

	pub async fn version(client: &Client) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_version", params).await?;
		Ok(value)
	}
}

pub mod chain {
	use super::*;

	pub async fn get_block(client: &Client, at: Option<H256>) -> Result<Option<ABlockDetailsRPC>, subxt::Error> {
		let params = rpc_params![at];
		let value = client.rpc_client.request("chain_getBlock", params).await?;
		Ok(value)
	}

	pub async fn get_block_hash(
		client: &Client,
		block_number: Option<BlockNumber>,
	) -> Result<Option<H256>, subxt::Error> {
		let params = rpc_params![block_number];
		let value = client.rpc_client.request("chain_getBlockHash", params).await?;
		Ok(value)
	}

	pub async fn get_finalized_head(client: &Client) -> Result<H256, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("chain_getFinalizedHead", params).await?;
		Ok(value)
	}

	pub async fn get_header(client: &Client, at: Option<H256>) -> Result<Option<AvailHeader>, subxt::Error> {
		let params = rpc_params![at];
		let value = client.rpc_client.request("chain_getHeader", params).await?;
		Ok(value)
	}
}

pub mod author {
	use super::*;

	pub async fn rotate_keys(client: &Client) -> Result<SessionKeys, subxt::Error> {
		let params = rpc_params![];
		let value: Bytes = client.rpc_client.request("author_rotateKeys", params).await?;
		let keys = utils::deconstruct_session_keys(value.0)?;
		Ok(keys)
	}

	pub async fn submit_extrinsic(client: &Client, extrinsic: &[u8]) -> Result<H256, subxt::Error> {
		let ext = std::format!("0x{}", hex::encode(extrinsic));
		let params = rpc_params![ext];
		let value: String = client.rpc_client.request("author_submitExtrinsic", params).await?;
		let value = utils::hex_string_to_h256(&value)?;
		Ok(value)
	}
}

pub mod state {
	use super::*;

	pub async fn get_runtime_version(client: &Client, at: Option<H256>) -> Result<RuntimeVersion, subxt::Error> {
		let params = rpc_params![at];
		let value = client.rpc_client.request("state_getRuntimeVersion", params).await?;
		Ok(value)
	}

	pub async fn call(client: &Client, method: &str, data: &[u8], at: Option<H256>) -> Result<String, subxt::Error> {
		let data = std::format!("0x{}", hex::encode(data));
		let params = rpc_params![method, data, at];
		let value = client.rpc_client.request("state_call", params).await?;
		Ok(value)
	}
}

pub mod kate {
	use super::*;

	pub async fn block_length(client: &Client, at: Option<H256>) -> Result<BlockLength, subxt::Error> {
		let params = rpc_params![at];
		let value = client.rpc_client.request("kate_blockLength", params).await?;
		Ok(value)
	}

	pub async fn query_data_proof(
		client: &Client,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, subxt::Error> {
		let params = rpc_params![transaction_index, at];
		let value = client.rpc_client.request("kate_queryDataProof", params).await?;
		Ok(value)
	}

	pub async fn query_proof(
		client: &Client,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<GDataProof>, subxt::Error> {
		let params = rpc_params![cells, at];
		let value = client.rpc_client.request("kate_queryProof", params).await?;
		Ok(value)
	}

	pub async fn query_rows(client: &Client, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, subxt::Error> {
		let params = rpc_params![rows, at];
		let value = client.rpc_client.request("kate_queryRows", params).await?;
		Ok(value)
	}
}

pub mod rpc {
	use super::*;

	#[derive(Debug, Default, Serialize, Deserialize)]
	pub struct RpcMethods {
		pub methods: Vec<String>,
	}

	pub async fn methods(client: &Client) -> Result<RpcMethods, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("rpc_methods", params).await?;
		Ok(value)
	}
}
