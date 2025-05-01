use crate::{
	avail::runtime_types::{da_runtime::primitives::SessionKeys, frame_system::limits::BlockLength},
	from_substrate::{NodeRole, PeerInfo, SyncState},
	utils::{self},
	ABlockDetailsRPC, AvailHeader, BlockNumber, Cell, Client, GDataProof, GRow, H256Ext, TransactionLocation, H256,
};
use avail_core::data_proof::ProofResponse;
use serde::{Deserialize, Serialize};
use subxt::{
	backend::legacy::rpc_methods::{BlockJustification, Bytes, RuntimeVersion, SystemHealth},
	ext::subxt_rpcs::rpc_params,
};
use subxt_core::config::{substrate::BlakeTwo256, Hasher};

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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RpcMethods {
	pub methods: Vec<String>,
}

#[derive(Clone)]
pub struct ChainBlock {
	pub block: ChainBlockBlock,
	pub justifications: Option<Vec<BlockJustification>>,
}

impl ChainBlock {
	pub fn has_transaction(&self, tx_hash: H256) -> Option<TransactionLocation> {
		for (i, tx) in self.block.extrinsics.iter().enumerate() {
			if BlakeTwo256::hash(tx) == tx_hash {
				return Some(TransactionLocation::from((tx_hash, i as u32)));
			}
		}

		None
	}
}

#[derive(Clone)]
pub struct ChainBlockBlock {
	/// The block header.
	pub header: AvailHeader,
	/// The accompanying extrinsics.
	pub extrinsics: Vec<Vec<u8>>,
}

impl Client {
	pub async fn rpc_system_account_next_index(&self, account: String) -> Result<u32, subxt::Error> {
		let params = rpc_params![account];
		let value = self.rpc_client.request("system_accountNextIndex", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_chain(&self) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_chain", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_chain_type(&self) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_chainType", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_health(&self) -> Result<SystemHealth, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_health", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_local_listen_addresses(&self) -> Result<Vec<String>, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_localListenAddresses", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_local_peer_id(&self) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_localPeerId", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_name(&self) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_name", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_node_roles(&self) -> Result<Vec<NodeRole>, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_nodeRoles", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_peers(&self) -> Result<Vec<PeerInfo>, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_peers", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_properties(&self) -> Result<SystemProperties, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_properties", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_sync_state(&self) -> Result<SyncState, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_syncState", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_version(&self) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_version", params).await?;
		Ok(value)
	}

	pub async fn rpc_chain_get_block(&self, at: Option<H256>) -> Result<Option<ChainBlock>, subxt::Error> {
		let params = rpc_params![at];
		let res: Option<ABlockDetailsRPC> = self.rpc_client.request("chain_getBlock", params).await?;
		let Some(res) = res else { return Ok(None) };

		let value = ChainBlock {
			block: ChainBlockBlock {
				header: res.block.header,
				extrinsics: res.block.extrinsics.into_iter().map(|x| x.0).collect(),
			},
			justifications: res.justifications,
		};
		Ok(Some(value))
	}

	pub async fn rpc_chain_get_block_hash(
		&self,
		block_height: Option<BlockNumber>,
	) -> Result<Option<H256>, subxt::Error> {
		let params = rpc_params![block_height];
		let value = self.rpc_client.request("chain_getBlockHash", params).await?;
		Ok(value)
	}

	pub async fn rpc_chain_get_finalized_head(&self) -> Result<H256, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("chain_getFinalizedHead", params).await?;
		Ok(value)
	}

	pub async fn rpc_chain_get_header(&self, at: Option<H256>) -> Result<Option<AvailHeader>, subxt::Error> {
		let params = rpc_params![at];
		let value = self.rpc_client.request("chain_getHeader", params).await?;
		Ok(value)
	}

	pub async fn rpc_author_rotate_keys(&self) -> Result<SessionKeys, subxt::Error> {
		let params = rpc_params![];
		let value: Bytes = self.rpc_client.request("author_rotateKeys", params).await?;
		let keys = utils::deconstruct_session_keys(value.0)?;
		Ok(keys)
	}

	pub async fn rpc_author_submit_extrinsic(&self, extrinsic: &[u8]) -> Result<H256, subxt::Error> {
		let ext = std::format!("0x{}", hex::encode(extrinsic));
		let params = rpc_params![ext];
		let value: String = self.rpc_client.request("author_submitExtrinsic", params).await?;
		let value = H256::from_str(&value)?;
		Ok(value)
	}

	pub async fn rpc_state_get_runtime_version(&self, at: Option<H256>) -> Result<RuntimeVersion, subxt::Error> {
		let params = rpc_params![at];
		let value = self.rpc_client.request("state_getRuntimeVersion", params).await?;
		Ok(value)
	}

	pub async fn rpc_state_call(&self, method: &str, data: &[u8], at: Option<H256>) -> Result<String, subxt::Error> {
		let data = std::format!("0x{}", hex::encode(data));
		let params = rpc_params![method, data, at];
		let value = self.rpc_client.request("state_call", params).await?;
		Ok(value)
	}

	pub async fn rpc_kate_block_length(&self, at: Option<H256>) -> Result<BlockLength, subxt::Error> {
		let params = rpc_params![at];
		let value = self.rpc_client.request("kate_blockLength", params).await?;
		Ok(value)
	}

	pub async fn rpc_kate_query_data_proof(
		&self,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, subxt::Error> {
		let params = rpc_params![transaction_index, at];
		let value = self.rpc_client.request("kate_queryDataProof", params).await?;
		Ok(value)
	}

	pub async fn rpc_kate_query_proof(
		&self,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<GDataProof>, subxt::Error> {
		let params = rpc_params![cells, at];
		let value = self.rpc_client.request("kate_queryProof", params).await?;
		Ok(value)
	}

	pub async fn rpc_kate_query_rows(&self, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, subxt::Error> {
		let params = rpc_params![rows, at];
		let value = self.rpc_client.request("kate_queryRows", params).await?;
		Ok(value)
	}

	pub async fn rpc_rpc_methods(&self) -> Result<RpcMethods, subxt::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("rpc_methods", params).await?;
		Ok(value)
	}
}
