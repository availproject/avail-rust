use crate::{Client, Error, RetryPolicy, UserError, block::shared::BlockContext, error_ops};
use avail_rust_core::{
	HasHeader, TransactionEventDecodable, avail,
	rpc::{self, AllowedEvents},
	types::{HashStringNumber, RuntimePhase, substrate::Weight},
};

/// Helper for retrieving events scoped to a specific block.
pub struct BlockEventsQuery {
	ctx: BlockContext,
}

impl BlockEventsQuery {
	/// Creates an event helper for a block hash/height.
	pub fn new(client: Client, at: impl Into<HashStringNumber>) -> Self {
		BlockEventsQuery { ctx: BlockContext::new(client, at.into()) }
	}

	/// Returns events emitted by a specific extrinsic index.
	pub async fn extrinsic(&self, tx_index: u32) -> Result<BlockEvents, Error> {
		let events = self.all(tx_index.into()).await?;
		Ok(events)
	}

	/// Returns system-level events that are not tied to extrinsics.
	pub async fn system(&self) -> Result<BlockEvents, Error> {
		let events = self.all(AllowedEvents::OnlyNonExtrinsics).await?;
		let events: Vec<BlockEvent> = events
			.0
			.into_iter()
			.filter(|x| x.phase.extrinsic_index().is_none())
			.collect();

		Ok(BlockEvents::new(events))
	}

	/// Returns all events for this block using the provided filter.
	pub async fn all(&self, allow_list: AllowedEvents) -> Result<BlockEvents, Error> {
		let phase_events = self.rpc(allow_list, true).await?;

		let mut result: Vec<BlockEvent> = Vec::new();
		for block_phase_event in phase_events {
			let phase = block_phase_event.phase;

			for phase_event in block_phase_event.events {
				result.push(BlockEvent::from_parts(phase_event, phase)?);
			}
		}

		Ok(BlockEvents::new(result))
	}

	/// Returns raw phase-grouped event data for this block.
	pub async fn rpc(&self, allow_list: AllowedEvents, fetch_data: bool) -> Result<Vec<rpc::PhaseEvents>, Error> {
		let at = self.ctx.hash_number()?;
		self.ctx.chain().events(at, allow_list, fetch_data).await
	}

	/// Sets retry behavior for event lookups.
	pub fn set_retry_policy(&mut self, value: RetryPolicy) {
		self.ctx.set_retry_policy(value);
	}

	/// Returns whether event queries retry after RPC errors.
	pub fn should_retry_on_error(&self) -> bool {
		self.ctx.should_retry_on_error()
	}

	/// Aggregates extrinsic weight from success/failed events.
	pub async fn extrinsic_weight(&self) -> Result<Weight, Error> {
		use avail::system::events::{ExtrinsicFailed, ExtrinsicSuccess};

		let mut weight = Weight::default();
		let events = self.all(AllowedEvents::OnlyExtrinsics).await?;
		for event in events.0 {
			if event.phase.extrinsic_index().is_none() {
				continue;
			}

			let header = (event.pallet_id, event.variant_id);
			if header == ExtrinsicSuccess::HEADER_INDEX {
				let e = ExtrinsicSuccess::from_event(event.data).map_err(|err| {
					Error::decode_with_op(
						error_ops::ErrorOperation::BlockEventsExtrinsicWeight,
						std::format!("Failed to decode ExtrinsicSuccess event: {}", err),
					)
				})?;
				weight.ref_time = weight.ref_time.saturating_add(e.dispatch_info.weight.ref_time);
				weight.proof_size = weight.proof_size.saturating_add(e.dispatch_info.weight.proof_size);
			} else if header == ExtrinsicFailed::HEADER_INDEX {
				let e = ExtrinsicFailed::from_event(event.data).map_err(|err| {
					Error::decode_with_op(
						error_ops::ErrorOperation::BlockEventsExtrinsicWeight,
						std::format!("Failed to decode ExtrinsicFailed event: {}", err),
					)
				})?;
				weight.ref_time = weight.ref_time.saturating_add(e.dispatch_info.weight.ref_time);
				weight.proof_size = weight.proof_size.saturating_add(e.dispatch_info.weight.proof_size);
			}
		}

		Ok(weight)
	}

	/// Returns the number of events emitted by this block.
	pub async fn event_count(&self) -> Result<usize, Error> {
		self.ctx.event_count().await
	}
}

/// Event emitted during block execution with contextual metadata.
#[derive(Debug, Clone)]
pub struct BlockEvent {
	/// Phase of block execution in which the event occurred.
	pub phase: RuntimePhase,
	/// Sequential index of the event within the phase.
	pub index: u32,
	/// Identifier of the emitting pallet.
	pub pallet_id: u8,
	/// Identifier of the variant inside the pallet.
	pub variant_id: u8,
	/// SCALE-encoded payload containing event data.
	pub data: String,
}

impl BlockEvent {
	/// Converts a raw phase event into a typed [`BlockEvent`].
	///
	/// Returns the converted event or an error when encoded data is missing.
	pub fn from_parts(event: rpc::RuntimeEvent, phase: RuntimePhase) -> Result<Self, Error> {
		let e = BlockEvent {
			index: event.index,
			pallet_id: event.pallet_id,
			variant_id: event.variant_id,
			data: event.data.clone(),
			phase,
		};

		Ok(e)
	}
}

/// Collection of block events with helpers for querying by header.
#[derive(Debug, Clone)]
pub struct BlockEvents(pub Vec<BlockEvent>);

impl BlockEvents {
	/// Wraps decoded events.
	///
	pub fn new(events: Vec<BlockEvent>) -> Self {
		Self(events)
	}

	/// Returns the first event matching the requested type.
	///
	pub fn first<T: HasHeader + codec::Decode>(&self) -> Option<T> {
		let event = self
			.0
			.iter()
			.find(|x| x.pallet_id == T::HEADER_INDEX.0 && x.variant_id == T::HEADER_INDEX.1);
		let event = event?;

		T::from_event(&event.data).ok()
	}

	/// Returns the last event matching the requested type.
	///
	pub fn last<T: HasHeader + codec::Decode>(&self) -> Option<T> {
		let event = self
			.0
			.iter()
			.rev()
			.find(|x| x.pallet_id == T::HEADER_INDEX.0 && x.variant_id == T::HEADER_INDEX.1);
		let event = event?;

		T::from_event(&event.data).ok()
	}

	/// Returns every event matching the requested type.
	///
	pub fn all<T: HasHeader + codec::Decode>(&self) -> Result<Vec<T>, Error> {
		let mut result = Vec::new();
		for event in &self.0 {
			if event.pallet_id != T::HEADER_INDEX.0 || event.variant_id != T::HEADER_INDEX.1 {
				continue;
			}

			let decoded = T::from_event(event.data.as_str()).map_err(|x| Error::User(UserError::Decoding(x)))?;
			result.push(decoded);
		}

		Ok(result)
	}

	/// Checks if an `ExtrinsicSuccess` event exists.
	///
	pub fn is_extrinsic_success_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicSuccess>()
	}

	/// Checks if an `ExtrinsicFailed` event exists.
	///
	pub fn is_extrinsic_failed_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicFailed>()
	}

	/// Returns whether a proxy call succeeded, when present.
	///
	pub fn proxy_executed_successfully(&self) -> Option<bool> {
		let executed = self.first::<avail::proxy::events::ProxyExecuted>()?;
		Some(executed.result.is_ok())
	}

	/// Returns whether a multisig call succeeded, when present.
	///
	pub fn multisig_executed_successfully(&self) -> Option<bool> {
		let executed = self.first::<avail::multisig::events::MultisigExecuted>()?;
		Some(executed.result.is_ok())
	}

	/// Returns true when at least one event of the given type exists.
	///
	pub fn is_present<T: HasHeader>(&self) -> bool {
		self.count::<T>() > 0
	}

	/// Returns true when the given pallet and variant combination appears.
	///
	pub fn is_present_parts(&self, pallet_id: u8, variant_id: u8) -> bool {
		self.count_parts(pallet_id, variant_id) > 0
	}

	/// Counts how many times the given event type appears.
	///
	pub fn count<T: HasHeader>(&self) -> u32 {
		self.count_parts(T::HEADER_INDEX.0, T::HEADER_INDEX.1)
	}

	/// Counts how many events match the pallet and variant combo.
	///
	pub fn count_parts(&self, pallet_id: u8, variant_id: u8) -> u32 {
		let mut count = 0;
		self.0.iter().for_each(|x| {
			if x.pallet_id == pallet_id && x.variant_id == variant_id {
				count += 1
			}
		});

		count
	}

	/// Returns the number of cached events.
	///
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Reports whether any events are cached.
	///
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}
