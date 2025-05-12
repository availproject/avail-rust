use crate::primitives::BlockHeight;
use crate::{
	avail,
	client::Client,
	config::*,
	error::RpcError,
	from_substrate::{NodeRole, PeerInfo, SyncState},
	AvailHeader, Cell, GDataProof, GRow,
};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use subxt_core::config::{substrate::BlakeTwo256, Hasher};
use subxt_rpcs::{
	methods::legacy::{BlockJustification, RuntimeVersion, SystemHealth},
	rpc_params,
};

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
	use crate::primitives::block::extrinsics::RuntimePhase;

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
		pub phase: RuntimePhase,
		pub emitted_index: EmittedIndex,
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

pub mod rpc_block_overview {
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
		pub transactions: Vec<TransactionData>,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct TransactionData {
		pub location: TransactionLocation,
		pub dispatch_index: DispatchIndex,
		pub signature: Option<TransactionSignature>,
		pub decoded: Option<String>,
		pub events: Option<Vec<Event>>,
	}

	#[derive(Debug, Default, Clone, Serialize, Deserialize)]
	pub struct TransactionSignature {
		pub ss58_address: Option<String>,
		pub nonce: u32,
		pub app_id: u32,
		pub mortality: Option<(u64, u64)>, // None means the tx is Immortal
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Event {
		pub index: u32,
		pub emitted_index: EmittedIndex,
		pub decoded: Option<String>,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct ConsensusEvent {
		pub phase: ConsensusEventPhase,
		pub emitted_index: EmittedIndex,
		pub decoded: Option<String>,
	}

	#[derive(Clone, Serialize, Deserialize)]
	pub struct Params {
		pub block_index: HashIndex,
		pub extension: ParamsExtension,
		pub filter: Filter,
	}

	impl Params {
		pub fn new(block_index: HashIndex) -> Self {
			Self {
				block_index,
				extension: ParamsExtension::default(),
				filter: Filter::default(),
			}
		}
	}

	#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
	pub struct ParamsExtension {
		pub enable_call_decoding: bool,
		pub fetch_events: bool,
		pub enable_event_decoding: bool,
		pub enable_consensus_event: bool,
	}

	#[derive(Debug, Default, Clone, Serialize, Deserialize)]
	pub struct Filter {
		pub transaction: TransactionFilterOptions,
		pub signature: SignatureFilterOptions,
	}

	#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
	pub enum BlockState {
		Included,
		Finalized,
		Discarded,
	}

	#[derive(Debug, Default, Clone, Serialize, Deserialize)]
	pub struct CallFilter {
		pub transaction: TransactionFilterOptions,
		pub signature: SignatureFilterOptions,
	}

	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub enum TransactionFilterOptions {
		All,
		TxHash(Vec<H256>),
		TxIndex(Vec<u32>),
		Pallet(Vec<u8>),
		PalletCall(Vec<DispatchIndex>),
		HasEvent(Vec<EmittedIndex>),
	}

	impl Default for TransactionFilterOptions {
		fn default() -> Self {
			Self::All
		}
	}

	#[derive(Debug, Default, Clone, Serialize, Deserialize)]
	pub struct SignatureFilterOptions {
		pub ss58_address: Option<String>,
		pub app_id: Option<u32>,
		pub nonce: Option<u32>,
	}

	#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
	pub enum ConsensusEventPhase {
		Finalization,
		Initialization,
	}
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PerDispatchClassU32 {
	pub normal: u32,
	pub operational: u32,
	pub mandatory: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockLength {
	pub max: PerDispatchClassU32,
	pub cols: u32,
	pub rows: u32,
	pub chunk_size: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofResponse {
	pub data_proof: DataProof,
	pub message: Option<avail::vector::types::AddressedMessage>,
}

/// Wrapper of `binary-merkle-tree::MerkleProof` with codec support.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataProof {
	pub roots: TxDataRoots,
	/// Proof items (does not contain the leaf hash, nor the root obviously).
	///
	/// This vec contains all inner node hashes necessary to reconstruct the root hash given the
	/// leaf hash.
	pub proof: Vec<H256>,
	/// Number of leaves in the original tree.
	///
	/// This is needed to detect a case where we have an odd number of leaves that "get promoted"
	/// to upper layers.
	pub number_of_leaves: u32,
	/// Index of the leaf the proof is for (0-based).
	pub leaf_index: u32,
	/// Leaf content.
	pub leaf: H256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxDataRoots {
	/// Global Merkle root
	pub data_root: H256,
	/// Merkle root hash of submitted data
	pub blob_root: H256,
	/// Merkle root of bridged data
	pub bridge_root: H256,
}

impl Client {
	pub async fn rpc_block_overview(
		&self,
		params: rpc_block_overview::Params,
	) -> Result<rpc_block_overview::Response, RpcError> {
		let params = rpc_params![params];
		let value = self.rpc_client.request("block_overview", params).await?;
		Ok(value)
	}

	pub async fn rpc_block_data(&self, params: rpc_block_data::Params) -> Result<rpc_block_data::Response, RpcError> {
		let params = rpc_params![params];
		let value = self.rpc_client.request("block_data", params).await?;
		Ok(value)
	}

	// TODO remove this is just for testing
	pub async fn rpc_error(&self) -> Result<u32, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_accountNextIndexx", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_account_next_index(&self, address: &str) -> Result<u32, RpcError> {
		let params = rpc_params![address];
		let value = self.rpc_client.request("system_accountNextIndex", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_chain(&self) -> Result<String, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_chain", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_chain_type(&self) -> Result<String, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_chainType", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_health(&self) -> Result<SystemHealth, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_health", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_local_listen_addresses(&self) -> Result<Vec<String>, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_localListenAddresses", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_local_peer_id(&self) -> Result<String, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_localPeerId", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_name(&self) -> Result<String, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_name", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_node_roles(&self) -> Result<Vec<NodeRole>, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_nodeRoles", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_peers(&self) -> Result<Vec<PeerInfo>, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_peers", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_properties(&self) -> Result<SystemProperties, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_properties", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_sync_state(&self) -> Result<SyncState, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_syncState", params).await?;
		Ok(value)
	}

	pub async fn rpc_system_version(&self) -> Result<String, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("system_version", params).await?;
		Ok(value)
	}

	pub async fn rpc_chain_get_block(&self, at: Option<H256>) -> Result<Option<ChainBlock>, RpcError> {
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

	pub async fn rpc_chain_get_block_hash(&self, block_height: Option<BlockHeight>) -> Result<Option<H256>, RpcError> {
		let params = rpc_params![block_height];
		let value = self.rpc_client.request("chain_getBlockHash", params).await?;
		Ok(value)
	}

	pub async fn rpc_chain_get_finalized_head(&self) -> Result<H256, RpcError> {
		raw::rpc_chain_get_finalized_head(&self.rpc_client).await
	}

	pub async fn rpc_chain_get_header(&self, at: Option<H256>) -> Result<Option<AvailHeader>, RpcError> {
		let params = rpc_params![at];
		let value = self.rpc_client.request("chain_getHeader", params).await?;
		Ok(value)
	}

	pub async fn rpc_author_rotate_keys(&self) -> Result<avail::utils::SessionKeys, RpcError> {
		let params = rpc_params![];
		let value: Vec<u8> = self.rpc_client.request("author_rotateKeys", params).await?;
		let keys = avail::utils::SessionKeys::try_from(value.as_slice())?;
		Ok(keys)
	}

	pub async fn rpc_author_submit_extrinsic(&self, extrinsic: &[u8]) -> Result<H256, RpcError> {
		let ext = std::format!("0x{}", hex::encode(extrinsic));
		let params = rpc_params![ext];
		let value: H256 = self.rpc_client.request("author_submitExtrinsic", params).await?;
		Ok(value)
	}

	pub async fn rpc_state_get_runtime_version(&self, at: Option<H256>) -> Result<RuntimeVersion, RpcError> {
		raw::rpc_state_get_runtime_version(&self.rpc_client, at).await
	}

	pub async fn rpc_state_call(&self, method: &str, data: &[u8], at: Option<H256>) -> Result<String, RpcError> {
		let data = std::format!("0x{}", hex::encode(data));
		let params = rpc_params![method, data, at];
		let value = self.rpc_client.request("state_call", params).await?;
		Ok(value)
	}

	pub async fn rpc_state_get_metadata(&self, at: Option<H256>) -> Result<Vec<u8>, RpcError> {
		raw::rpc_state_get_metadata(&self.rpc_client, at).await
	}

	pub async fn rpc_state_get_storage(&self, key: Vec<u8>, at: Option<H256>) -> Result<Option<Vec<u8>>, RpcError> {
		let key = hex::encode(key);
		let params = rpc_params![key, at];
		let value: Option<String> = self.rpc_client.request("state_getStorage", params).await?;
		let Some(value) = value else { return Ok(None) };
		let value = hex::decode(value.trim_start_matches("0x"));
		let value = value.map_err(|e| RpcError::from(e.to_string()))?;
		Ok(Some(value))
	}

	pub async fn rpc_kate_block_length(&self, at: Option<H256>) -> Result<BlockLength, RpcError> {
		let params = rpc_params![at];
		let value = self.rpc_client.request("kate_blockLength", params).await?;
		Ok(value)
	}

	pub async fn rpc_kate_query_data_proof(
		&self,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, RpcError> {
		let params = rpc_params![transaction_index, at];
		let value = self.rpc_client.request("kate_queryDataProof", params).await?;
		Ok(value)
	}

	pub async fn rpc_kate_query_proof(&self, cells: Vec<Cell>, at: Option<H256>) -> Result<Vec<GDataProof>, RpcError> {
		let params = rpc_params![cells, at];
		let value = self.rpc_client.request("kate_queryProof", params).await?;
		Ok(value)
	}

	pub async fn rpc_kate_query_rows(&self, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, RpcError> {
		let params = rpc_params![rows, at];
		let value = self.rpc_client.request("kate_queryRows", params).await?;
		Ok(value)
	}

	pub async fn rpc_rpc_methods(&self) -> Result<RpcMethods, RpcError> {
		let params = rpc_params![];
		let value = self.rpc_client.request("rpc_methods", params).await?;
		Ok(value)
	}

	pub async fn rpc_chainspec_v1_genesishash(&self) -> Result<H256, RpcError> {
		raw::rpc_chainspec_v1_genesishash(&self.rpc_client).await
	}
}

pub mod raw {
	use super::*;
	use crate::prelude::H256Ext;
	use subxt_rpcs::RpcClient;

	pub async fn rpc_chainspec_v1_genesishash(client: &RpcClient) -> Result<H256, RpcError> {
		let params = rpc_params![];
		let value: String = client.request("chainSpec_v1_genesisHash", params).await?;
		Ok(H256::from_str(&value).unwrap())
	}

	pub async fn rpc_state_get_metadata(client: &RpcClient, at: Option<H256>) -> Result<Vec<u8>, RpcError> {
		let params = rpc_params![at];
		let value: String = client.request("state_getMetadata", params).await?;

		Ok(hex::decode(value.trim_start_matches("0x")).unwrap())
	}

	pub async fn rpc_chain_get_finalized_head(client: &RpcClient) -> Result<H256, RpcError> {
		let params = rpc_params![];
		let value = client.request("chain_getFinalizedHead", params).await?;
		Ok(value)
	}

	pub async fn rpc_state_get_runtime_version(
		client: &RpcClient,
		at: Option<H256>,
	) -> Result<RuntimeVersion, RpcError> {
		let params = rpc_params![at];
		let value = client.request("state_getRuntimeVersion", params).await?;
		Ok(value)
	}
}
