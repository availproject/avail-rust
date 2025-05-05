use crate::{
	avail::runtime_types::{da_runtime::primitives::SessionKeys, frame_system::limits::BlockLength},
	client::Client,
	config::*,
	error::RpcError,
	from_substrate::{NodeRole, PeerInfo, SyncState},
	utils::{self},
	AvailHeader, Cell, GDataProof, GRow,
};
use primitive_types::H256;
// use avail_core::data_proof::ProofResponse;
use serde::{Deserialize, Serialize};
use subxt::{
	backend::legacy::rpc_methods::{BlockJustification, Bytes, RuntimeVersion, SystemHealth},
	ext::subxt_rpcs::rpc_params,
};
use subxt_core::config::{substrate::BlakeTwo256, Hasher};

/// Arbitrary properties defined in chain spec as a JSON object
pub type SystemProperties = serde_json::map::Map<String, serde_json::Value>;

#[derive(Clone, Serialize, Deserialize)]
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
}

#[derive(Default, Serialize, Deserialize)]
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

pub mod rpc_block_data {
	pub use super::*;

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Response {
		pub value: Block,
		pub debug_execution_time: u64,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]

	pub struct Block {
		pub block_id: BlockId,
		pub block_state: BlockState,
		pub calls: Option<Vec<CallData>>,
		pub events: Option<Vec<EventData>>,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct CallData {
		pub tx_location: TransactionLocation,
		pub dispatch_index: DispatchIndex,
		pub call: String,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct EventData {
		// (pallet id, event id)
		pub emitted_index: EmittedIndex,
		pub phase: RuntimePhase,
		pub event: String,
	}

	#[derive(Clone, Serialize, Deserialize)]
	pub struct Params {
		pub block_index: HashIndex,
		pub fetch_calls: bool,
		pub fetch_events: bool,
		pub call_filter: CallFilter,
		pub event_filter: EventFilter,
	}

	impl Params {
		pub fn new(block_index: HashIndex) -> Self {
			Self {
				block_index,
				fetch_calls: false,
				fetch_events: false,
				call_filter: CallFilter::default(),
				event_filter: EventFilter::default(),
			}
		}
	}

	#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
	pub enum BlockState {
		Included,
		Finalized,
		Discarded,
	}

	#[derive(Default, Clone, Serialize, Deserialize)]
	pub struct CallFilter {
		pub transaction: TransactionFilterOptions,
		pub signature: SignatureFilterOptions,
	}

	#[derive(Clone, Serialize, Deserialize)]
	pub enum TransactionFilterOptions {
		All,
		TxHash(Vec<H256>),
		TxIndex(Vec<u32>),
		Pallet(Vec<u8>),
		PalletCall(Vec<DispatchIndex>),
	}

	impl Default for TransactionFilterOptions {
		fn default() -> Self {
			Self::All
		}
	}

	#[derive(Default, Clone, Serialize, Deserialize)]
	pub struct SignatureFilterOptions {
		pub ss58_address: Option<String>,
		pub app_id: Option<u32>,
		pub nonce: Option<u32>,
	}

	#[derive(Default, Clone, Serialize, Deserialize)]
	pub struct EventFilter {
		pub phase: PhaseFilterOptions,
		pub pallet: PalletFilterOptions,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub enum PhaseFilterOptions {
		All,
		TxIndex(Vec<u32>),
		OnlyConsensus,
	}

	impl Default for PhaseFilterOptions {
		fn default() -> Self {
			Self::All
		}
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub enum PalletFilterOptions {
		All,
		Pallet(u8),
		Combination(Vec<DispatchIndex>),
	}

	impl Default for PalletFilterOptions {
		fn default() -> Self {
			Self::All
		}
	}
}

impl Client {
	pub async fn rpc_block_data(
		&self,
		params: rpc_block_data::Params,
	) -> Result<rpc_block_data::Response, subxt_rpcs::Error> {
		let params = rpc_params![params];
		let value = self.rpc_client.request("block_data", params).await?;
		Ok(value)
	}

	// TODO remove this is just for testing
	pub async fn rpc_error(&self) -> Result<u32, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_accountNextIndexx", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_account_next_index(&self, address: &str) -> Result<u32, subxt_rpcs::Error> {
		let params = rpc_params![address];
		let value = self.rpc_client.request("system_accountNextIndex", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_chain(&self) -> Result<String, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_chain", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_chain_type(&self) -> Result<String, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_chainType", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_health(&self) -> Result<SystemHealth, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_health", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_local_listen_addresses(&self) -> Result<Vec<String>, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_localListenAddresses", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_local_peer_id(&self) -> Result<String, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_localPeerId", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_name(&self) -> Result<String, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_name", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_node_roles(&self) -> Result<Vec<NodeRole>, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_nodeRoles", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_peers(&self) -> Result<Vec<PeerInfo>, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_peers", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_properties(&self) -> Result<SystemProperties, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_properties", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_sync_state(&self) -> Result<SyncState, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_syncState", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_version(&self) -> Result<String, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_version", params).await?;
		Ok(value)
	}

	pub async fn rpc_chain_get_block(&self, at: Option<H256>) -> Result<Option<ChainBlock>, subxt_rpcs::Error> {
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
	) -> Result<Option<H256>, subxt_rpcs::Error> {
		let params = rpc_params![block_height];
		let value = self.rpc_client.request("chain_getBlockHash", params).await?;
		Ok(value)
	}

	pub async fn rpc_chain_get_finalized_head(&self) -> Result<H256, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("chain_getFinalizedHead", params).await?;
		Ok(value)
	}

	pub async fn rpc_chain_get_header(&self, at: Option<H256>) -> Result<Option<AvailHeader>, subxt_rpcs::Error> {
		let params = rpc_params![at];
		let value = self.rpc_client.request("chain_getHeader", params).await?;
		Ok(value)
	}

	pub async fn rpc_author_rotate_keys(&self) -> Result<SessionKeys, RpcError> {
		let params = rpc_params![];
		let value: Bytes = self.rpc_client.request("author_rotateKeys", params).await?;
		let keys = utils::deconstruct_session_keys(value.0)?;
		Ok(keys)
	}

	pub async fn rpc_author_submit_extrinsic(&self, extrinsic: &[u8]) -> Result<H256, subxt_rpcs::Error> {
		let ext = std::format!("0x{}", hex::encode(extrinsic));
		let params = rpc_params![ext];
		let value: H256 = self.rpc_client.request("author_submitExtrinsic", params).await?;
		Ok(value)
	}

	pub async fn rpc_state_get_runtime_version(&self, at: Option<H256>) -> Result<RuntimeVersion, subxt_rpcs::Error> {
		let params = rpc_params![at];
		let value = self.rpc_client.request("state_getRuntimeVersion", params).await?;
		Ok(value)
	}

	pub async fn rpc_state_call(
		&self,
		method: &str,
		data: &[u8],
		at: Option<H256>,
	) -> Result<String, subxt_rpcs::Error> {
		let data = std::format!("0x{}", hex::encode(data));
		let params = rpc_params![method, data, at];
		let value = self.rpc_client.request("state_call", params).await?;
		Ok(value)
	}

	pub async fn rpc_kate_block_length(&self, at: Option<H256>) -> Result<BlockLength, subxt_rpcs::Error> {
		let params = rpc_params![at];
		let value = self.rpc_client.request("kate_blockLength", params).await?;
		Ok(value)
	}

	/* 	pub async fn rpc_kate_query_data_proof(
		&self,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, RpcError> {
		let params = rpc_params![transaction_index, at];
		let value = self.rpc_client.request("kate_queryDataProof", params).await?;
		Ok(value)
	} */

	pub async fn rpc_kate_query_proof(
		&self,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<GDataProof>, subxt_rpcs::Error> {
		let params = rpc_params![cells, at];
		let value = self.rpc_client.request("kate_queryProof", params).await?;
		Ok(value)
	}

	pub async fn rpc_kate_query_rows(&self, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, subxt_rpcs::Error> {
		let params = rpc_params![rows, at];
		let value = self.rpc_client.request("kate_queryRows", params).await?;
		Ok(value)
	}

	pub async fn rpc_rpc_methods(&self) -> Result<RpcMethods, subxt_rpcs::Error> {
		let params = rpc_params![];
		let value = self.rpc_client.request("rpc_methods", params).await?;
		Ok(value)
	}
}
