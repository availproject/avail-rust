use crate::{
	client::Client,
	subxt_rpcs::methods::legacy::{RuntimeVersion, SystemHealth},
};
use core::{
	rpc,
	rpc::{
		kate::{BlockLength, Cell, GDataProof, GRow, ProofResponse},
		substrate::{
			BlockWithJustifications, NodeRole, PeerInfo, RpcMethods, SessionKeys, SyncState, SystemProperties,
		},
	},
	AvailHeader, H256,
};

/* #[derive(Clone, Serialize, Deserialize)]
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
	pub async fn state(client: &Client, tx_hash: &H256, finalized: bool) -> Result<Vec<TransactionState>, RpcError> {
		let params = rpc_params![tx_hash, finalized];
		let value = client.rpc_client.request("transaction_state", params).await?;
		Ok(value)
	}
} */

impl Client {
	pub async fn rpc_block_overview(
		&self,
		params: rpc::block::block_overview::Params,
	) -> Result<rpc::block::block_overview::Response, core::Error> {
		Ok(rpc::block::block_overview(&self.rpc_client, params).await?)
	}

	pub async fn rpc_block_data(
		&self,
		params: rpc::block::block_data::Params,
	) -> Result<rpc::block::block_data::Response, core::Error> {
		Ok(rpc::block::block_data(&self.rpc_client, params).await?)
	}

	pub async fn rpc_system_account_next_index(&self, address: &str) -> Result<u32, core::Error> {
		Ok(rpc::substrate::system_account_next_index(&self.rpc_client, address).await?)
	}

	pub async fn rpc_system_chain(&self) -> Result<String, core::Error> {
		Ok(rpc::substrate::system_chain(&self.rpc_client).await?)
	}

	pub async fn rpc_system_chain_type(&self) -> Result<String, core::Error> {
		Ok(rpc::substrate::system_chain_type(&self.rpc_client).await?)
	}

	pub async fn rpc_system_health(&self) -> Result<SystemHealth, core::Error> {
		Ok(rpc::substrate::system_health(&self.rpc_client).await?)
	}

	pub async fn rpc_system_local_listen_addresses(&self) -> Result<Vec<String>, core::Error> {
		Ok(rpc::substrate::system_local_listen_addresses(&self.rpc_client).await?)
	}

	pub async fn rpc_system_local_peer_id(&self) -> Result<String, core::Error> {
		Ok(rpc::substrate::system_local_peer_id(&self.rpc_client).await?)
	}

	pub async fn rpc_system_name(&self) -> Result<String, core::Error> {
		Ok(rpc::substrate::system_name(&self.rpc_client).await?)
	}

	pub async fn rpc_system_node_roles(&self) -> Result<Vec<NodeRole>, core::Error> {
		Ok(rpc::substrate::system_node_roles(&self.rpc_client).await?)
	}

	pub async fn rpc_system_peers(&self) -> Result<Vec<PeerInfo>, core::Error> {
		Ok(rpc::substrate::system_peers(&self.rpc_client).await?)
	}

	pub async fn rpc_system_properties(&self) -> Result<SystemProperties, core::Error> {
		Ok(rpc::substrate::system_properties(&self.rpc_client).await?)
	}

	pub async fn rpc_system_sync_state(&self) -> Result<SyncState, core::Error> {
		Ok(rpc::substrate::system_sync_state(&self.rpc_client).await?)
	}

	pub async fn rpc_system_version(&self) -> Result<String, core::Error> {
		Ok(rpc::substrate::system_version(&self.rpc_client).await?)
	}

	pub async fn rpc_chain_get_block(&self, at: Option<H256>) -> Result<Option<BlockWithJustifications>, core::Error> {
		Ok(rpc::substrate::chain_get_block(&self.rpc_client, at).await?)
	}

	pub async fn rpc_chain_get_block_hash(&self, block_height: Option<u32>) -> Result<Option<H256>, core::Error> {
		Ok(rpc::substrate::chain_get_block_hash(&self.rpc_client, block_height).await?)
	}

	pub async fn rpc_chain_get_finalized_head(&self) -> Result<H256, core::Error> {
		Ok(rpc::substrate::chain_get_finalized_head(&self.rpc_client).await?)
	}

	pub async fn rpc_chain_get_header(&self, at: Option<H256>) -> Result<Option<AvailHeader>, core::Error> {
		Ok(rpc::substrate::chain_get_header(&self.rpc_client, at).await?)
	}

	pub async fn rpc_author_rotate_keys(&self) -> Result<SessionKeys, core::Error> {
		Ok(rpc::substrate::author_rotate_keys(&self.rpc_client).await?)
	}

	pub async fn rpc_author_submit_extrinsic(&self, extrinsic: &[u8]) -> Result<H256, core::Error> {
		Ok(rpc::substrate::author_submit_extrinsic(&self.rpc_client, extrinsic).await?)
	}

	pub async fn rpc_state_get_runtime_version(&self, at: Option<H256>) -> Result<RuntimeVersion, core::Error> {
		Ok(rpc::substrate::state_get_runtime_version(&self.rpc_client, at).await?)
	}

	pub async fn rpc_state_call(&self, method: &str, data: &[u8], at: Option<H256>) -> Result<String, core::Error> {
		Ok(rpc::substrate::state_call(&self.rpc_client, method, data, at).await?)
	}

	pub async fn rpc_state_get_metadata(&self, at: Option<H256>) -> Result<Vec<u8>, core::Error> {
		Ok(rpc::substrate::state_get_metadata(&self.rpc_client, at).await?)
	}

	pub async fn rpc_state_get_storage(&self, key: &str, at: Option<H256>) -> Result<Option<Vec<u8>>, core::Error> {
		Ok(rpc::substrate::state_get_storage(&self.rpc_client, key, at).await?)
	}

	pub async fn rpc_rpc_methods(&self) -> Result<RpcMethods, core::Error> {
		Ok(rpc::substrate::rpc_methods(&self.rpc_client).await?)
	}

	pub async fn rpc_chainspec_v1_genesishash(&self) -> Result<H256, core::Error> {
		Ok(rpc::substrate::chainspec_v1_genesishash(&self.rpc_client).await?)
	}

	pub async fn rpc_kate_block_length(&self, at: Option<H256>) -> Result<BlockLength, core::Error> {
		Ok(rpc::kate::kate_block_length(&self.rpc_client, at).await?)
	}

	pub async fn rpc_kate_query_data_proof(
		&self,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, core::Error> {
		Ok(rpc::kate::kate_query_data_proof(&self.rpc_client, transaction_index, at).await?)
	}

	pub async fn rpc_kate_query_proof(
		&self,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<GDataProof>, core::Error> {
		Ok(rpc::kate::kate_query_proof(&self.rpc_client, cells, at).await?)
	}

	pub async fn rpc_kate_query_rows(&self, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, core::Error> {
		Ok(rpc::kate::kate_query_rows(&self.rpc_client, rows, at).await?)
	}
}
