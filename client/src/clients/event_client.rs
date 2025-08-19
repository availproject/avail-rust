use crate::{clients::Client, subxt_core::events::Phase};
use avail_rust_core::{H256, HashNumber, decoded_events::OpaqueEvent, rpc::system::fetch_events_v1_types as Types};

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

	pub fn event_bytes(&self) -> &[u8] {
		&self.bytes.0
	}

	pub fn event_data(&self) -> &[u8] {
		self.bytes.event_data()
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
		block_id: HashNumber,
		tx_index: u32,
	) -> Result<Option<Vec<Types::RuntimeEvent>>, avail_rust_core::Error> {
		let builder = self
			.builder()
			.tx_index(tx_index)
			.enable_encoding(true)
			.enable_decoding(false)
			.retry_on_error(true);

		let result = builder.fetch(block_id).await?;
		Ok(result.first().map(|x| x.events.clone()))
	}

	/// Function to fetch blocks events in a efficient manner.
	pub fn builder(&self) -> BlockEventsBuilder {
		BlockEventsBuilder::new(self.client.clone())
	}

	/// Use this function in case where `transaction_events` or `block_events` do not work.
	/// Both mentioned functions require the runtime to have a specific runtime api available which
	/// older blocks (runtime) do not have.
	pub async fn historical_block_events(&self, at: H256) -> Result<Vec<Event>, avail_rust_core::Error> {
		use crate::{config::AvailConfig, subxt_core::events::Events};

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
			let Ok(raw) = raw else {
				continue;
			};
			let mut bytes: Vec<u8> = Vec::with_capacity(raw.field_bytes().len() + 2);
			bytes.push(raw.pallet_index());
			bytes.push(raw.variant_index());
			bytes.append(&mut raw.field_bytes().to_vec());

			let Ok(bytes) = OpaqueEvent::try_from(bytes) else {
				continue;
			};

			let value = Event { phase: raw.phase(), bytes, topics: raw.topics().to_vec() };
			result.push(value);
		}

		Ok(result)
	}
}

#[derive(Clone)]
pub struct BlockEventsBuilder {
	client: Client,
	filter: Types::Filter,
	enable_encoding: bool,
	enable_decoding: bool,
	retry_on_error: bool,
}

impl BlockEventsBuilder {
	pub fn new(client: Client) -> Self {
		Self {
			client,
			filter: Default::default(),
			enable_encoding: false,
			enable_decoding: false,
			retry_on_error: false,
		}
	}

	pub fn reset(mut self) -> Self {
		self.filter = Default::default();
		self.enable_encoding = false;
		self.enable_decoding = false;
		self.retry_on_error = false;
		self
	}

	pub fn filter(mut self, value: Types::Filter) -> Self {
		self.filter = value;
		self
	}

	pub fn all(mut self) -> Self {
		self.filter = Types::Filter::All;
		self
	}

	pub fn only_extrinsics(mut self) -> Self {
		self.filter = Types::Filter::OnlyExtrinsics;
		self
	}

	pub fn no_extrinsics(mut self) -> Self {
		self.filter = Types::Filter::OnlyNonExtrinsics;
		self
	}

	pub fn tx_index(self, value: u32) -> Self {
		self.tx_indexes(vec![value])
	}

	pub fn tx_indexes(mut self, value: Vec<u32>) -> Self {
		self.filter = Types::Filter::Only(value);
		self
	}

	pub fn enable_encoding(mut self, value: bool) -> Self {
		self.enable_encoding = value;
		self
	}

	pub fn enable_decoding(mut self, value: bool) -> Self {
		self.enable_decoding = value;
		self
	}

	pub fn retry_on_error(mut self, value: bool) -> Self {
		self.retry_on_error = value;
		self
	}

	pub async fn fetch(&self, block_id: HashNumber) -> Result<Types::Output, avail_rust_core::Error> {
		let block_hash = match block_id {
			HashNumber::Hash(hash) => hash,
			HashNumber::Number(height) => {
				let hash = self.client.block_hash_ext(height, self.retry_on_error, false).await?;
				hash.ok_or(avail_rust_core::Error::from("Failed to fetch block hash"))?
			},
		};

		let options = Types::Options {
			filter: Some(self.filter.clone()),
			enable_encoding: Some(self.enable_encoding),
			enable_decoding: Some(self.enable_encoding),
		};

		self.client
			.rpc_api()
			.system_fetch_events_v1_ext(block_hash, options.clone(), self.retry_on_error)
			.await
	}
}
