use avail_rust_core::{
	ext::{
		codec::Decode,
		subxt_rpcs::{
			client::RpcParams,
			methods::legacy::{RuntimeVersion, SystemHealth},
		},
	},
	grandpa::GrandpaJustification,
	rpc::{
		self,
		kate::{BlockLength, Cell, GDataProof, GRow, ProofResponse},
		substrate::{NodeRole, PeerInfo, RpcMethods, SessionKeys, SyncState, SystemProperties},
		BlockWithJustifications,
	},
	AvailHeader, H256,
};

use crate::Client;

#[derive(Clone)]
pub struct RpcAPI {
	client: Client,
}

impl RpcAPI {
	pub fn new(client: Client) -> Self {
		Self { client }
	}

	pub async fn call<T: serde::de::DeserializeOwned>(
		&self,
		method: &str,
		params: RpcParams,
	) -> Result<T, avail_rust_core::Error> {
		Ok(rpc::substrate::call_raw(&self.client.rpc_client, method, params).await?)
	}

	pub async fn block_overview(
		&self,
		params: rpc::block::block_overview::Params,
	) -> Result<rpc::block::block_overview::Response, avail_rust_core::Error> {
		Ok(rpc::block::block_overview(&self.client.rpc_client, params).await?)
	}

	pub async fn block_data(
		&self,
		params: rpc::block::block_data::Params,
	) -> Result<rpc::block::block_data::Response, avail_rust_core::Error> {
		Ok(rpc::block::block_data(&self.client.rpc_client, params).await?)
	}

	pub async fn system_account_next_index(&self, address: &str) -> Result<u32, avail_rust_core::Error> {
		Ok(rpc::substrate::system_account_next_index(&self.client.rpc_client, address).await?)
	}

	pub async fn system_chain(&self) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::substrate::system_chain(&self.client.rpc_client).await?)
	}

	pub async fn system_chain_type(&self) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::substrate::system_chain_type(&self.client.rpc_client).await?)
	}

	pub async fn system_health(&self) -> Result<SystemHealth, avail_rust_core::Error> {
		Ok(rpc::substrate::system_health(&self.client.rpc_client).await?)
	}

	pub async fn system_local_listen_addresses(&self) -> Result<Vec<String>, avail_rust_core::Error> {
		Ok(rpc::substrate::system_local_listen_addresses(&self.client.rpc_client).await?)
	}

	pub async fn system_local_peer_id(&self) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::substrate::system_local_peer_id(&self.client.rpc_client).await?)
	}

	pub async fn system_name(&self) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::substrate::system_name(&self.client.rpc_client).await?)
	}

	pub async fn system_node_roles(&self) -> Result<Vec<NodeRole>, avail_rust_core::Error> {
		Ok(rpc::substrate::system_node_roles(&self.client.rpc_client).await?)
	}

	pub async fn system_peers(&self) -> Result<Vec<PeerInfo>, avail_rust_core::Error> {
		Ok(rpc::substrate::system_peers(&self.client.rpc_client).await?)
	}

	pub async fn system_properties(&self) -> Result<SystemProperties, avail_rust_core::Error> {
		Ok(rpc::substrate::system_properties(&self.client.rpc_client).await?)
	}

	pub async fn system_sync_state(&self) -> Result<SyncState, avail_rust_core::Error> {
		Ok(rpc::substrate::system_sync_state(&self.client.rpc_client).await?)
	}

	pub async fn system_version(&self) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::substrate::system_version(&self.client.rpc_client).await?)
	}

	pub async fn chain_get_block(
		&self,
		at: Option<H256>,
	) -> Result<Option<BlockWithJustifications>, avail_rust_core::Error> {
		Ok(rpc::substrate::chain_get_block(&self.client.rpc_client, at).await?)
	}

	pub async fn chain_get_block_hash(
		&self,
		block_height: Option<u32>,
	) -> Result<Option<H256>, avail_rust_core::Error> {
		Ok(rpc::substrate::chain_get_block_hash(&self.client.rpc_client, block_height).await?)
	}

	pub async fn chain_get_finalized_head(&self) -> Result<H256, avail_rust_core::Error> {
		Ok(rpc::substrate::chain_get_finalized_head(&self.client.rpc_client).await?)
	}

	pub async fn chain_get_header(&self, at: Option<H256>) -> Result<Option<AvailHeader>, avail_rust_core::Error> {
		Ok(rpc::substrate::chain_get_header(&self.client.rpc_client, at).await?)
	}

	pub async fn author_rotate_keys(&self) -> Result<SessionKeys, avail_rust_core::Error> {
		Ok(rpc::substrate::author_rotate_keys(&self.client.rpc_client).await?)
	}

	pub async fn author_submit_extrinsic(&self, extrinsic: &[u8]) -> Result<H256, avail_rust_core::Error> {
		Ok(rpc::substrate::author_submit_extrinsic(&self.client.rpc_client, extrinsic).await?)
	}

	pub async fn state_get_runtime_version(&self, at: Option<H256>) -> Result<RuntimeVersion, avail_rust_core::Error> {
		Ok(rpc::substrate::state_get_runtime_version(&self.client.rpc_client, at).await?)
	}

	pub async fn state_call(
		&self,
		method: &str,
		data: &[u8],
		at: Option<H256>,
	) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::substrate::state_call(&self.client.rpc_client, method, data, at).await?)
	}

	pub async fn state_get_metadata(&self, at: Option<H256>) -> Result<Vec<u8>, avail_rust_core::Error> {
		Ok(rpc::substrate::state_get_metadata(&self.client.rpc_client, at).await?)
	}

	pub async fn state_get_storage(
		&self,
		key: &str,
		at: Option<H256>,
	) -> Result<Option<Vec<u8>>, avail_rust_core::Error> {
		Ok(rpc::substrate::state_get_storage(&self.client.rpc_client, key, at).await?)
	}

	pub async fn rpc_methods(&self) -> Result<RpcMethods, avail_rust_core::Error> {
		Ok(rpc::substrate::rpc_methods(&self.client.rpc_client).await?)
	}

	pub async fn chainspec_v1_genesishash(&self) -> Result<H256, avail_rust_core::Error> {
		Ok(rpc::substrate::chainspec_v1_genesishash(&self.client.rpc_client).await?)
	}

	pub async fn kate_block_length(&self, at: Option<H256>) -> Result<BlockLength, avail_rust_core::Error> {
		Ok(rpc::kate::kate_block_length(&self.client.rpc_client, at).await?)
	}

	pub async fn kate_query_data_proof(
		&self,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, avail_rust_core::Error> {
		Ok(rpc::kate::kate_query_data_proof(&self.client.rpc_client, transaction_index, at).await?)
	}

	pub async fn kate_query_proof(
		&self,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<GDataProof>, avail_rust_core::Error> {
		Ok(rpc::kate::kate_query_proof(&self.client.rpc_client, cells, at).await?)
	}

	pub async fn kate_query_rows(&self, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, avail_rust_core::Error> {
		Ok(rpc::kate::kate_query_rows(&self.client.rpc_client, rows, at).await?)
	}

	pub async fn grandpa_block_justification(
		&self,
		at: u32,
	) -> Result<Option<GrandpaJustification>, avail_rust_core::Error> {
		let result = rpc::grandpa::grandpa_block_justification(&self.client.rpc_client, at).await?;
		let Some(result) = result else { return Ok(None) };

		let justification = GrandpaJustification::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;
		Ok(Some(justification))
	}
}
