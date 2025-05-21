use crate::{client::Client, config::AvailConfig, subxt_core::events::Phase};
use core::{avail::RuntimeEvent, H256};

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

	// Block Events
	pub async fn block_events(&self, at: H256) -> Result<Vec<Event>, core::Error> {
		use crate::subxt_core::events::Events;

		let entries = self
			.client
			.rpc_state_get_storage(EVENTS_STORAGE_ADDRESS, Some(at))
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

			let value = Event {
				phase: raw.phase(),
				bytes,
				topics: raw.topics().to_vec(),
			};
			result.push(value);
		}

		Ok(result)
	}

	// Transaction Events
	pub async fn transaction_events(&self, tx_index: u32, at: H256) -> Result<Vec<Event>, core::Error> {
		use crate::subxt_core::events::{Events, Phase};

		let entries = self
			.client
			.rpc_state_get_storage(EVENTS_STORAGE_ADDRESS, Some(at))
			.await?;
		let Some(event_bytes) = entries else {
			return Ok(Vec::new());
		};

		let mut result: Vec<Event> = Vec::with_capacity(10);
		let raw_events = Events::<AvailConfig>::decode_from(event_bytes, self.client.online_client().metadata());
		for raw in raw_events.iter() {
			let Ok(raw) = raw else { todo!() };
			match raw.phase() {
				Phase::ApplyExtrinsic(x) => {
					if tx_index > x {
						continue;
					}
					if tx_index < x {
						break;
					}
				},
				_ => continue,
			};
			let mut bytes: Vec<u8> = Vec::with_capacity(raw.field_bytes().len() + 2);
			bytes.push(raw.pallet_index());
			bytes.push(raw.variant_index());
			bytes.append(&mut raw.field_bytes().to_vec());

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
