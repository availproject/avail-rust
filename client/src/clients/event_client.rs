use crate::{clients::Client, subxt_core::events::Phase};
use avail_rust_core::{
	H256, avail::RuntimeEvent, decoded_events::OpaqueEvent, rpc::system::fetch_events_v1_types as Types,
};

pub const EVENTS_STORAGE_ADDRESS: &str = "0x26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7";

#[derive(Debug, Clone)]
pub struct Event {
	pub phase: Phase,
	// [Pallet_index, Variant_index, Event_data...]
	pub bytes: OpaqueEvent,
	pub topics: Vec<H256>,
}

impl Event {
	pub fn emitted_index(&self) -> (u8, u8) {
		(self.bytes.pallet_index(), self.bytes.variant_index())
	}

	pub fn pallet_index(&self) -> u8 {
		self.bytes.pallet_index()
	}

	pub fn variant_index(&self) -> u8 {
		self.bytes.variant_index()
	}

	pub fn event_data(&self) -> &[u8] {
		&self.bytes.event_data()
	}
}

impl TryFrom<Event> for RuntimeEvent {
	type Error = crate::codec::Error;

	fn try_from(value: Event) -> Result<Self, Self::Error> {
		Self::try_from(&value)
	}
}

impl TryFrom<&Event> for RuntimeEvent {
	type Error = crate::codec::Error;

	fn try_from(value: &Event) -> Result<Self, Self::Error> {
		RuntimeEvent::try_from(value.bytes.0.as_slice())
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

	/// Function to fetch transaction events in a efficient manner.
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

	/// Function to fetch blocks events in a efficient manner.
	pub async fn block_events(&self, params: Types::Params, at: H256) -> Result<Types::Output, avail_rust_core::Error> {
		self.client.rpc_api().system_fetch_events_v1(params, at).await
	}

	/// Use this function in case where `transaction_events` or `block_events` do not work.
	/// Both mentioned functions require the runtime to have a specific runtime api available which
	/// older blocks (runtime) do not have.
	pub async fn historical_block_events(&self, at: H256) -> Result<Vec<Event>, avail_rust_core::Error> {
		use crate::config::AvailConfig;
		use crate::subxt_core::events::Events;

		let entries = self
			.client
			.rpc_api()
			.state_get_storage(EVENTS_STORAGE_ADDRESS, Some(at))
			.await?;
		let Some(event_bytes) = entries else {
			return Ok(Vec::new());
		};

		let mut result: Vec<Event> = Vec::with_capacity(5);
		let raw_events = Events::<AvailConfig>::decode_from(event_bytes, self.client.online_client().metadata());
		for raw in raw_events.iter() {
			let Ok(raw) = raw else { todo!() };
			let mut bytes: Vec<u8> = Vec::with_capacity(raw.field_bytes().len() + 2);
			bytes.push(raw.pallet_index());
			bytes.push(raw.variant_index());
			bytes.append(&mut raw.field_bytes().to_vec());

			let Ok(bytes) = OpaqueEvent::try_from(bytes) else {
				continue;
			};

			let value = Event {
				phase: raw.phase(),
				bytes,
				topics: raw.topics().to_vec(),
			};
			result.push(value);
		}

		Ok(result)
	}
}
