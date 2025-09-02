use crate::{clients::Client, subxt_core::events::Phase};
use avail_rust_core::{
	H256, HasHeader, HashNumber, TransactionEventDecodable, avail, decoded_events::OpaqueEvent,
	rpc::system::fetch_events,
};

pub const EVENTS_STORAGE_ADDRESS: &str = "0x26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7";

pub struct TransactionEvent {
	pub index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub data: String,
}

pub struct TransactionEvents {
	pub events: Vec<TransactionEvent>,
}

impl TransactionEvents {
	pub fn new(events: Vec<TransactionEvent>) -> Self {
		Self { events }
	}

	pub fn find<T: HasHeader + codec::Decode>(&self) -> Option<T> {
		let event = self
			.events
			.iter()
			.find(|x| x.pallet_id == T::HEADER_INDEX.0 && x.variant_id == T::HEADER_INDEX.1);
		let Some(event) = event else {
			return None;
		};

		T::decode_hex_event(&event.data)
	}

	pub fn is_extrinsic_success_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicSuccess>()
	}

	pub fn is_extrinsic_failed_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicFailed>()
	}

	pub fn proxy_executed_successfully(&self) -> Option<bool> {
		let executed = self.find::<avail::proxy::events::ProxyExecuted>()?;
		return Some(executed.result.is_ok());
	}

	pub fn multisig_executed_successfully(&self) -> Option<bool> {
		let executed = self.find::<avail::multisig::events::MultisigExecuted>()?;
		return Some(executed.result.is_ok());
	}

	pub fn is_present<T: HasHeader>(&self) -> bool {
		self.count::<T>() > 0
	}

	pub fn is_present_parts(&self, pallet_id: u8, variant_id: u8) -> bool {
		self.count_parts(pallet_id, variant_id) > 0
	}

	pub fn count<T: HasHeader>(&self) -> u32 {
		self.count_parts(T::HEADER_INDEX.0, T::HEADER_INDEX.1)
	}

	pub fn count_parts(&self, pallet_id: u8, variant_id: u8) -> u32 {
		let mut count = 0;
		self.events.iter().for_each(|x| {
			if x.pallet_id == pallet_id && x.variant_id == variant_id {
				count += 1
			}
		});

		count
	}
}

#[derive(Debug, Clone)]
pub struct HistoricalEvent {
	pub phase: Phase,
	// [Pallet_index, Variant_index, Event_data...]
	pub bytes: OpaqueEvent,
	pub topics: Vec<H256>,
}

impl HistoricalEvent {
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
	) -> Result<Option<TransactionEvents>, avail_rust_core::Error> {
		let builder = self
			.builder()
			.tx_index(tx_index)
			.enable_encoding(true)
			.enable_decoding(false)
			.retry_on_error(true);

		let result = builder.fetch(block_id).await?;
		let Some(first) = result.first() else {
			return Ok(None);
		};

		let mut tx_events = Vec::with_capacity(first.events.len());
		for event in &first.events {
			let Some(data) = &event.encoded else {
				return Err("Fetch events endpoint returned with an event without data.".into());
			};
			tx_events.push(TransactionEvent {
				data: data.clone(),
				index: event.index,
				pallet_id: event.emitted_index.0,
				variant_id: event.emitted_index.1,
			});
		}

		Ok(Some(TransactionEvents::new(tx_events)))
	}

	/// Function to fetch blocks events in a efficient manner.
	pub fn builder(&self) -> BlockEventsBuilder {
		BlockEventsBuilder::new(self.client.clone())
	}

	/// Use this function in case where `transaction_events` or `block_events` do not work.
	/// Both mentioned functions require the runtime to have a specific runtime api available which
	/// older blocks (runtime) do not have.
	pub async fn historical_block_events(&self, at: H256) -> Result<Vec<HistoricalEvent>, avail_rust_core::Error> {
		use crate::{config::AvailConfig, subxt_core::events::Events};

		let entries = self
			.client
			.rpc_api()
			.state_get_storage(EVENTS_STORAGE_ADDRESS, Some(at))
			.await?;
		let Some(event_bytes) = entries else {
			return Ok(Vec::new());
		};

		let mut result: Vec<HistoricalEvent> = Vec::with_capacity(5);
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

			let value = HistoricalEvent { phase: raw.phase(), bytes, topics: raw.topics().to_vec() };
			result.push(value);
		}

		Ok(result)
	}
}

#[derive(Clone)]
pub struct BlockEventsBuilder {
	client: Client,
	filter: fetch_events::Filter,
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

	pub fn filter(mut self, value: fetch_events::Filter) -> Self {
		self.filter = value;
		self
	}

	pub fn all(mut self) -> Self {
		self.filter = fetch_events::Filter::All;
		self
	}

	pub fn only_extrinsics(mut self) -> Self {
		self.filter = fetch_events::Filter::OnlyExtrinsics;
		self
	}

	pub fn no_extrinsics(mut self) -> Self {
		self.filter = fetch_events::Filter::OnlyNonExtrinsics;
		self
	}

	pub fn tx_index(self, value: u32) -> Self {
		self.tx_indexes(vec![value])
	}

	pub fn tx_indexes(mut self, value: Vec<u32>) -> Self {
		self.filter = fetch_events::Filter::Only(value);
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

	pub async fn fetch(&self, block_id: HashNumber) -> Result<Vec<fetch_events::PhaseEvents>, avail_rust_core::Error> {
		let block_hash = match block_id {
			HashNumber::Hash(hash) => hash,
			HashNumber::Number(height) => {
				let hash = self.client.block_hash_ext(height, self.retry_on_error, false).await?;
				hash.ok_or(avail_rust_core::Error::from("Failed to fetch block hash"))?
			},
		};

		let options = fetch_events::Options {
			filter: Some(self.filter.clone()),
			enable_encoding: Some(self.enable_encoding),
			enable_decoding: Some(self.enable_encoding),
		};

		self.client
			.rpc_api()
			.system_fetch_events_v1_ext(block_hash, options, self.retry_on_error)
			.await
	}
}
