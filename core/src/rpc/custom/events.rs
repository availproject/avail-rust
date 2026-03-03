use crate::{HashNumber, rpc::Error, types::RuntimePhase};
use serde::{Deserialize, Serialize};
use subxt_rpcs::{RpcClient, rpc_params};

pub async fn fetch_events(
	client: &RpcClient,
	at: HashNumber,
	allow_list: AllowedEvents,
	fetch_data: bool,
) -> Result<Vec<PhaseEvents>, Error> {
	let params = rpc_params![at, allow_list, fetch_data];
	let value: Vec<PhaseEvents> = client.request("custom_events", params).await?;
	Ok(value)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub enum AllowedEvents {
	#[default]
	All = 0,
	OnlyExtrinsics = 1,
	OnlyNonExtrinsics = 2,
	Only(Vec<u32>) = 3,
}

impl From<u32> for AllowedEvents {
	fn from(value: u32) -> Self {
		Self::Only(vec![value])
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PhaseEvents {
	pub phase: RuntimePhase,
	pub events: Vec<RuntimeEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RuntimeEvent {
	pub index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub data: String,
}
