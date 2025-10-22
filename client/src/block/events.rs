use crate::{Client, Error, UserError, block::shared::BlockContext};
use avail_rust_core::{
	HasHeader, RpcError, TransactionEventDecodable, avail,
	rpc::{self, BlockPhaseEvent},
	types::{HashStringNumber, RuntimePhase, substrate::Weight},
};

pub struct Events {
	ctx: BlockContext,
}

impl Events {
	/// Creates an event view for the given block.
	///
	/// # Parameters
	/// - `client`: RPC client used to fetch event data.
	/// - `block_id`: Identifier convertible into `HashStringNumber`.
	///
	/// # Returns
	/// - `Self`: Helper for retrieving events scoped to the block.
	pub fn new(client: Client, block_id: impl Into<HashStringNumber>) -> Self {
		Events { ctx: BlockContext::new(client, block_id.into()) }
	}

	/// Returns events emitted by a specific extrinsic index.
	///
	/// # Parameters
	/// - `tx_index`: Index of the extrinsic whose events should be returned.
	///
	/// # Returns
	/// - `Ok(AllEvents)`: Wrapper around events emitted by the extrinsic (may be empty).
	/// - `Err(Error)`: RPC retrieval or event decoding failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry as configured.
	pub async fn extrinsic(&self, tx_index: u32) -> Result<AllEvents, Error> {
		let events = self.all(tx_index.into()).await?;
		Ok(AllEvents::new(events))
	}

	/// Returns system-level events that are not tied to extrinsics.
	///
	/// # Returns
	/// - `Ok(AllEvents)`: Wrapper around system events (may be empty).
	/// - `Err(Error)`: RPC retrieval or event decoding failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry as configured.
	pub async fn system(&self) -> Result<AllEvents, Error> {
		let events = self.all(rpc::EventFilter::OnlyNonExtrinsics).await?;
		let events: Vec<Event> = events
			.into_iter()
			.filter(|x| x.phase.extrinsic_index().is_none())
			.collect();

		Ok(AllEvents::new(events))
	}

	/// Fetches all events for the block using the given filter.
	///
	/// # Parameters
	/// - `filter`: Filter describing which phases or extrinsics to include.
	///
	/// # Returns
	/// - `Ok(Vec<Event>)`: Zero or more events matching the filter.
	/// - `Err(Error)`: RPC retrieval or event decoding failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry as configured.
	pub async fn all(&self, filter: rpc::EventFilter) -> Result<Vec<Event>, Error> {
		let opts = rpc::EventOpts {
			filter: Some(filter),
			enable_encoding: Some(true),
			enable_decoding: Some(false),
		};

		let block_id = self.ctx.block_id.clone();
		let chain = self.ctx.chain();
		let block_phase_events = chain.system_fetch_events(block_id, opts).await?;

		let mut result: Vec<Event> = Vec::new();
		for block_phase_event in block_phase_events {
			let phase = block_phase_event.phase;

			for mut phase_event in block_phase_event.events {
				let Some(data) = phase_event.encoded_data.take() else {
					return Err(
						RpcError::ExpectedData("The node did not return encoded data for this event.".into()).into()
					);
				};

				let all_event = Event {
					index: phase_event.index,
					pallet_id: phase_event.pallet_id,
					variant_id: phase_event.variant_id,
					data,
					phase,
				};
				result.push(all_event);
			}
		}

		Ok(result)
	}

	/// Fetches raw event data with full RPC control.
	///
	/// # Parameters
	/// - `opts`: RPC options specifying filters and encoding preferences.
	///
	/// # Returns
	/// - `Ok(Vec<BlockPhaseEvent>)`: Raw events grouped by block phase.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry as configured.
	pub async fn raw(&self, opts: rpc::EventOpts) -> Result<Vec<BlockPhaseEvent>, Error> {
		let block_id = self.ctx.block_id.clone();
		let chain = self.ctx.chain();
		let result = chain.system_fetch_events(block_id, opts).await?;

		Ok(result)
	}

	/// Overrides retry behaviour for event lookups.
	///
	/// # Parameters
	/// - `value`: Retry override (`Some(true)` to force retries, `Some(false)` to disable, `None` to inherit).
	///
	/// # Returns
	/// - `()`: The new retry preference is stored.
	///
	/// # Side Effects
	/// - Updates internal state so future RPC requests honour the override.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.ctx.set_retry_on_error(value);
	}

	/// Reports whether event queries retry after RPC errors.
	///
	/// # Returns
	/// - `true`: Retries are enabled either explicitly or via the client default.
	/// - `false`: Retries are disabled.
	pub fn should_retry_on_error(&self) -> bool {
		self.ctx.should_retry_on_error()
	}

	/// Aggregates weight consumed by extrinsics using emitted events.
	///
	/// # Returns
	/// - `Ok(Weight)`: Summed weights derived from `ExtrinsicSuccess` and `ExtrinsicFailed` events.
	/// - `Err(Error)`: Event retrieval or decoding failed.
	///
	/// # Side Effects
	/// - Issues RPC requests for event data and may retry as configured.
	pub async fn extrinsic_weight(&self) -> Result<Weight, Error> {
		use avail::system::events::{ExtrinsicFailed, ExtrinsicSuccess};

		let mut weight = Weight::default();
		let events = self.all(rpc::EventFilter::OnlyExtrinsics).await?;
		for event in events {
			if event.phase.extrinsic_index().is_none() {
				continue;
			}

			let header = (event.pallet_id, event.variant_id);
			if header == ExtrinsicSuccess::HEADER_INDEX {
				let e = ExtrinsicSuccess::from_event(event.data).map_err(Error::Other)?;
				weight.ref_time += e.dispatch_info.weight.ref_time;
				weight.proof_size += e.dispatch_info.weight.proof_size;
			} else if header == ExtrinsicFailed::HEADER_INDEX {
				let e = ExtrinsicFailed::from_event(event.data).map_err(Error::Other)?;
				weight.ref_time += e.dispatch_info.weight.ref_time;
				weight.proof_size += e.dispatch_info.weight.proof_size;
			}
		}

		Ok(weight)
	}

	/// Counts events emitted by this block.
	///
	/// # Returns
	/// - `Ok(u32)`: Number of events emitted in the block.
	/// - `Err(Error)`: RPC retrieval failed.
	///
	/// # Side Effects
	/// - Issues an RPC request and may retry as configured.
	pub async fn event_count(&self) -> Result<u32, Error> {
		let block_id = self.ctx.block_id.clone();
		let chain = self.ctx.chain();
		chain.block_event_count(block_id).await
	}
}

#[derive(Debug, Clone)]
pub struct Event {
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

#[derive(Debug, Clone)]
pub struct AllEvents {
	/// Collection of decoded events preserved in original order.
	pub events: Vec<Event>,
}

impl AllEvents {
	/// Wraps decoded events.
	///
	/// # Parameters
	/// - `events`: Collection of decoded events to wrap.
	///
	/// # Returns
	/// - `Self`: Wrapper exposing helper methods for event queries.
	pub fn new(events: Vec<Event>) -> Self {
		Self { events }
	}

	/// Returns the first event matching the requested type.
	///
	/// # Returns
	/// - `Some(T)`: First event decoded as the requested type.
	/// - `None`: No matching event was found or decoding failed.
	pub fn first<T: HasHeader + codec::Decode>(&self) -> Option<T> {
		let event = self
			.events
			.iter()
			.find(|x| x.pallet_id == T::HEADER_INDEX.0 && x.variant_id == T::HEADER_INDEX.1);
		let event = event?;

		T::from_event(&event.data).ok()
	}

	/// Returns the last event matching the requested type.
	///
	/// # Returns
	/// - `Some(T)`: Last event decoded as the requested type.
	/// - `None`: No matching event was found or decoding failed.
	pub fn last<T: HasHeader + codec::Decode>(&self) -> Option<T> {
		let event = self
			.events
			.iter()
			.rev()
			.find(|x| x.pallet_id == T::HEADER_INDEX.0 && x.variant_id == T::HEADER_INDEX.1);
		let event = event?;

		T::from_event(&event.data).ok()
	}

	/// Returns every event matching the requested type.
	///
	/// # Returns
	/// - `Ok(Vec<T>)`: Zero or more events decoded as the requested type.
	/// - `Err(Error)`: Event decoding failed.
	pub fn all<T: HasHeader + codec::Decode>(&self) -> Result<Vec<T>, Error> {
		let mut result = Vec::new();
		for event in &self.events {
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
	/// # Returns
	/// - `true`: At least one `ExtrinsicSuccess` event is present.
	/// - `false`: No such events were recorded.
	pub fn is_extrinsic_success_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicSuccess>()
	}

	/// Checks if an `ExtrinsicFailed` event exists.
	///
	/// # Returns
	/// - `true`: At least one `ExtrinsicFailed` event is present.
	/// - `false`: No such events were recorded.
	pub fn is_extrinsic_failed_present(&self) -> bool {
		self.is_present::<avail::system::events::ExtrinsicFailed>()
	}

	/// Returns whether a proxy call succeeded, when present.
	///
	/// # Returns
	/// - `Some(true)`: A proxy call executed successfully.
	/// - `Some(false)`: A proxy call executed but failed.
	/// - `None`: No proxy execution event was recorded.
	pub fn proxy_executed_successfully(&self) -> Option<bool> {
		let executed = self.first::<avail::proxy::events::ProxyExecuted>()?;
		Some(executed.result.is_ok())
	}

	/// Returns whether a multisig call succeeded, when present.
	///
	/// # Returns
	/// - `Some(true)`: A multisig call executed successfully.
	/// - `Some(false)`: A multisig call executed but failed.
	/// - `None`: No multisig execution event was recorded.
	pub fn multisig_executed_successfully(&self) -> Option<bool> {
		let executed = self.first::<avail::multisig::events::MultisigExecuted>()?;
		Some(executed.result.is_ok())
	}

	/// Returns true when at least one event of the given type exists.
	///
	/// # Returns
	/// - `true`: At least one matching event exists.
	/// - `false`: No matching events were recorded.
	pub fn is_present<T: HasHeader>(&self) -> bool {
		self.count::<T>() > 0
	}

	/// Returns true when the given pallet and variant combination appears.
	///
	/// # Parameters
	/// - `pallet_id`: Target pallet identifier.
	/// - `variant_id`: Target variant identifier.
	///
	/// # Returns
	/// - `true`: At least one matching event exists.
	/// - `false`: No matching events were recorded.
	pub fn is_present_parts(&self, pallet_id: u8, variant_id: u8) -> bool {
		self.count_parts(pallet_id, variant_id) > 0
	}

	/// Counts how many times the given event type appears.
	///
	/// # Returns
	/// - `u32`: Number of matching events recorded.
	pub fn count<T: HasHeader>(&self) -> u32 {
		self.count_parts(T::HEADER_INDEX.0, T::HEADER_INDEX.1)
	}

	/// Counts how many events match the pallet and variant combo.
	///
	/// # Parameters
	/// - `pallet_id`: Target pallet identifier.
	/// - `variant_id`: Target variant identifier.
	///
	/// # Returns
	/// - `u32`: Number of matching events recorded.
	pub fn count_parts(&self, pallet_id: u8, variant_id: u8) -> u32 {
		let mut count = 0;
		self.events.iter().for_each(|x| {
			if x.pallet_id == pallet_id && x.variant_id == variant_id {
				count += 1
			}
		});

		count
	}

	/// Returns the number of cached events.
	///
	/// # Returns
	/// - `usize`: Total events stored in the wrapper.
	pub fn len(&self) -> usize {
		self.events.len()
	}

	/// Reports whether any events are cached.
	///
	/// # Returns
	/// - `true`: The wrapper contains no events.
	/// - `false`: At least one event is stored.
	pub fn is_empty(&self) -> bool {
		self.events.is_empty()
	}
}
