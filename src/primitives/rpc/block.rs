use crate::primitives::block::extrinsics::RuntimePhase;
use crate::primitives::config::TransactionLocation;
use crate::primitives::{BlockId, DispatchIndex, EmittedIndex, HashIndex};
use primitive_types::H256;
use serde::Deserialize;
use serde::Serialize;
use subxt_rpcs::rpc_params;
use subxt_rpcs::RpcClient;

pub mod block_data {
	pub use super::*;

	#[derive(Debug, Clone, Deserialize)]
	pub struct Response {
		pub value: Block,
		pub debug_execution_time: u64,
	}

	#[derive(Debug, Clone, Deserialize)]

	pub struct Block {
		pub block_id: BlockId,
		pub block_state: BlockState,
		pub calls: Option<Vec<CallData>>,
		pub events: Option<Vec<EventData>>,
	}

	#[derive(Debug, Clone, Deserialize)]
	pub struct CallData {
		pub tx_location: TransactionLocation,
		pub dispatch_index: DispatchIndex,
		pub call: String,
	}

	#[derive(Debug, Clone, Deserialize)]
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

	#[derive(Debug, Clone, Copy, Deserialize)]
	pub enum BlockState {
		Included,
		Finalized,
		Discarded,
	}

	#[derive(Default, Serialize, Clone, Deserialize)]
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

	#[derive(Default, Serialize, Clone, Deserialize)]
	pub struct SignatureFilterOptions {
		pub ss58_address: Option<String>,
		pub app_id: Option<u32>,
		pub nonce: Option<u32>,
	}

	#[derive(Default, Serialize, Clone, Deserialize)]
	pub struct EventFilter {
		pub phase: PhaseFilterOptions,
		pub pallet: PalletFilterOptions,
	}

	#[derive(Debug, Serialize, Clone, Deserialize)]
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

	#[derive(Debug, Serialize, Clone, Deserialize)]
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

pub mod block_overview {
	pub use super::*;

	#[derive(Debug, Clone, Deserialize)]
	pub struct Response {
		pub value: Block,
		pub debug_execution_time: u64,
	}

	#[derive(Debug, Clone, Deserialize)]

	pub struct Block {
		pub block_id: BlockId,
		pub block_state: BlockState,
		pub transactions: Vec<TransactionData>,
	}

	#[derive(Debug, Clone, Deserialize)]
	pub struct TransactionData {
		pub location: TransactionLocation,
		pub dispatch_index: DispatchIndex,
		pub signature: Option<TransactionSignature>,
		pub decoded: Option<String>,
		pub events: Option<Vec<Event>>,
	}

	#[derive(Debug, Default, Clone, Deserialize)]
	pub struct TransactionSignature {
		pub ss58_address: Option<String>,
		pub nonce: u32,
		pub app_id: u32,
		pub mortality: Option<(u64, u64)>, // None means the tx is Immortal
	}

	#[derive(Debug, Clone, Deserialize)]
	pub struct Event {
		pub index: u32,
		pub emitted_index: EmittedIndex,
		pub decoded: Option<String>,
	}

	#[derive(Debug, Clone, Deserialize)]
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

	#[derive(Debug, Default, Serialize, Clone, Copy, Deserialize)]
	pub struct ParamsExtension {
		pub enable_call_decoding: bool,
		pub fetch_events: bool,
		pub enable_event_decoding: bool,
		pub enable_consensus_event: bool,
	}

	#[derive(Debug, Default, Serialize, Clone, Deserialize)]
	pub struct Filter {
		pub transaction: TransactionFilterOptions,
		pub signature: SignatureFilterOptions,
	}

	#[derive(Debug, Clone, Copy, Deserialize)]
	pub enum BlockState {
		Included,
		Finalized,
		Discarded,
	}

	#[derive(Debug, Default, Clone, Deserialize)]
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

	#[derive(Debug, Clone, Copy, Deserialize)]
	pub enum ConsensusEventPhase {
		Finalization,
		Initialization,
	}
}

pub async fn block_overview(
	client: &RpcClient,
	params: block_overview::Params,
) -> Result<block_overview::Response, subxt_rpcs::Error> {
	let params = rpc_params![params];
	let value = client.request("block_overview", params).await?;
	Ok(value)
}

pub async fn block_data(
	client: &RpcClient,
	params: block_data::Params,
) -> Result<block_data::Response, subxt_rpcs::Error> {
	let params = rpc_params![params];
	let value = client.request("block_data", params).await?;
	Ok(value)
}
