use crate::decoded_events::RuntimePhase;
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use subxt_rpcs::{RpcClient, rpc_params};

pub async fn fetch_events_v1(
	client: &RpcClient,
	at: H256,
	opts: &Options,
) -> Result<Vec<BlockPhaseEvent>, subxt_rpcs::Error> {
	let params = rpc_params![at, opts];
	let value: Vec<RpcPhaseEvents> = client.request("system_fetchEventsV1", params).await?;
	Ok(value.into_iter().map(BlockPhaseEvent::from).collect())
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Options {
	pub filter: Option<Filter>,
	pub enable_encoding: Option<bool>,
	pub enable_decoding: Option<bool>,
}

impl Options {
	pub fn new(filter: Option<Filter>, enable_encoding: Option<bool>, enable_decoding: Option<bool>) -> Self {
		Self { filter, enable_encoding, enable_decoding }
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

impl From<u32> for Filter {
	fn from(value: u32) -> Self {
		Self::Only(vec![value])
	}
}

impl From<Vec<u32>> for Filter {
	fn from(value: Vec<u32>) -> Self {
		Self::Only(value)
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockPhaseEvent {
	pub phase: RuntimePhase,
	pub events: Vec<PhaseEvent>,
}

impl From<RpcPhaseEvents> for BlockPhaseEvent {
	fn from(value: RpcPhaseEvents) -> Self {
		Self {
			phase: value.phase,
			events: value.events.into_iter().map(PhaseEvent::from).collect(),
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhaseEvent {
	pub index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub encoded_data: Option<String>,
	pub decoded_data: Option<String>,
}

impl From<RuntimeEvent> for PhaseEvent {
	fn from(value: RuntimeEvent) -> Self {
		Self {
			index: value.index,
			pallet_id: value.emitted_index.0,
			variant_id: value.emitted_index.1,
			encoded_data: value.encoded,
			decoded_data: value.decoded,
		}
	}
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct RpcPhaseEvents {
	pub phase: RuntimePhase,
	pub events: Vec<RuntimeEvent>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct RuntimeEvent {
	pub index: u32,
	// (Pallet Id, Event Id)
	pub emitted_index: (u8, u8),
	pub encoded: Option<String>,
	pub decoded: Option<String>,
}
