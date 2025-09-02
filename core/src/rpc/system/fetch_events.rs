use crate::decoded_events::RuntimePhase;
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use subxt_rpcs::{RpcClient, rpc_params};

pub async fn fetch_events_v1(
	client: &RpcClient,
	at: H256,
	options: &Options,
) -> Result<Vec<PhaseEvents>, subxt_rpcs::Error> {
	let params = rpc_params![at, options];
	let value = client.request("system_fetchEventsV1", params).await?;
	Ok(value)
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

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PhaseEvents {
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
