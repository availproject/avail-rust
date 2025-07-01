use crate::{clients::Client, subxt_core::events::Phase};
use avail_rust_core::{H256, avail::RuntimeEvent, rpc::system::fetch_events_v1_types as Types};

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
	) -> Result<Option<Types::GroupedRuntimeEvents>, avail_rust_core::Error> {
		let params = Types::Params::new(
			Some(Types::Filter::Only(vec![tx_id])),
			Some(enable_encoding),
			Some(enable_decoding),
		);

		let mut result = self.client.rpc_api().system_fetch_events_v1(params, at).await?;
		if result.is_empty() {
			return Ok(None);
		}
		Ok(Some(result.remove(0)))
	}

	pub async fn block_events(&self, params: Types::Params, at: H256) -> Result<Types::Output, avail_rust_core::Error> {
		self.client.rpc_api().system_fetch_events_v1(params, at).await
	}
}
