use crate::{clients::Client, subxt_core::events::Phase};
use avail_rust_core::{
	avail::RuntimeEvent,
	rpc::{self, system::fetch_events_v1_types},
	H256,
};

pub const EVENTS_STORAGE_ADDRESS: &str = "0x26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7";

#[derive(Debug, Clone)]
pub struct Event {
	pub phase: Phase,
	// [Pallet_index, Variant_index, Event_data...]
	pub bytes: Vec<u8>,
	pub topics: Vec<H256>,
}

impl Event {
	pub fn emitted_index(&self) -> (u8, u8) {
		(self.bytes[0], self.bytes[1])
	}

	pub fn pallet_index(&self) -> u8 {
		self.bytes[0]
	}

	pub fn variant_index(&self) -> u8 {
		self.bytes[1]
	}

	pub fn event_data(&self) -> &[u8] {
		&self.bytes[2..]
	}
}

impl TryFrom<&Event> for RuntimeEvent {
	type Error = crate::codec::Error;

	fn try_from(value: &Event) -> Result<Self, Self::Error> {
		value.bytes.as_slice().try_into()
	}
}

#[derive(Clone)]
pub struct EventClient {
	client: Client,
}

impl EventClient {
	pub fn new(client: Client) -> Self {
		Self { client }
	}

	pub async fn transaction_events(
		&self,
		tx_id: u32,
		enable_encoding: bool,
		enable_decoding: bool,
		at: H256,
	) -> Result<Option<fetch_events_v1_types::GroupedRuntimeEvents>, avail_rust_core::Error> {
		use fetch_events_v1_types::{Filter, Params};
		let params = Params::new()
			.with_encoding(enable_encoding)
			.with_decoding(enable_decoding)
			.with_filter(Filter::Only(vec![tx_id]));

		let mut result = rpc::system::fetch_events_v1(&self.client.rpc_client, params, at).await?;
		if result.len() == 0 {
			return Ok(None);
		}
		Ok(Some(result.remove(0)))
	}

	pub async fn block_events(
		&self,
		params: fetch_events_v1_types::Params,
		at: H256,
	) -> Result<fetch_events_v1_types::Output, avail_rust_core::Error> {
		Ok(rpc::system::fetch_events_v1(&self.client.rpc_client, params, at).await?)
	}
}
