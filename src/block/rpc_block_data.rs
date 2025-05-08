use crate::{
	client::{rpc::rpc_block_data, Client},
	config::{BlockId, DispatchIndex, EmittedIndex, HashIndex, RuntimePhase, TransactionLocation},
	error::RpcError,
	primitives::block::extrinsics::UncheckedEvent,
	AppUncheckedExtrinsic,
};
use codec::Decode;

#[derive(Clone)]
pub struct BlockBuilder {
	params: rpc_block_data::Params,
}

impl BlockBuilder {
	pub fn new(block_index: HashIndex) -> Self {
		Self {
			params: rpc_block_data::Params::new(block_index),
		}
	}

	pub fn block_index(mut self, value: HashIndex) -> Self {
		self.params.block_index = value;
		self
	}

	pub fn fetch_calls(mut self, value: bool) -> Self {
		self.params.fetch_calls = value;
		self
	}

	pub fn fetch_events(mut self, value: bool) -> Self {
		self.params.fetch_events = value;
		self
	}

	pub fn call_filter(mut self, value: rpc_block_data::CallFilter) -> Self {
		self.params.call_filter = value;
		self
	}

	pub fn event_filter(mut self, value: rpc_block_data::EventFilter) -> Self {
		self.params.event_filter = value;
		self
	}

	pub async fn build(&self, client: &Client) -> Result<Block, RpcError> {
		let response = client.rpc_block_data(self.params.clone()).await.map(|x| x.value)?;
		let calls = if let Some(list) = response.calls {
			Some(list.into_iter().map(CallData::from).collect())
		} else {
			None
		};

		let events = if let Some(list) = response.events {
			Some(list.into_iter().map(EventData::from).collect())
		} else {
			None
		};

		Ok(Block {
			block_id: response.block_id,
			block_state: response.block_state,
			calls,
			events,
		})
	}
}

#[derive(Debug, Clone)]
pub struct Block {
	pub block_id: BlockId,
	pub block_state: rpc_block_data::BlockState,
	pub calls: Option<Vec<CallData>>,
	pub events: Option<Vec<EventData>>,
}

#[derive(Debug, Clone)]
pub struct CallData {
	pub tx_location: TransactionLocation,
	pub dispatch_index: DispatchIndex,
	// None if we failed to decode it
	pub call: Option<AppUncheckedExtrinsic>,
}

impl From<rpc_block_data::CallData> for CallData {
	fn from(value: rpc_block_data::CallData) -> Self {
		let call = match hex::decode(value.call.trim_start_matches("0x")) {
			Ok(x) => AppUncheckedExtrinsic::decode(&mut x.as_slice()).ok(),
			Err(_) => None,
		};

		Self {
			tx_location: value.tx_location,
			dispatch_index: value.dispatch_index,
			call,
		}
	}
}

#[derive(Debug, Clone)]
pub struct EventData {
	pub phase: RuntimePhase,
	pub emitted_index: EmittedIndex,
	// None if we failed to decode it
	pub event: Option<UncheckedEvent>,
}

impl From<rpc_block_data::EventData> for EventData {
	fn from(value: rpc_block_data::EventData) -> Self {
		let event = match hex::decode(value.event.trim_start_matches("0x")) {
			Ok(x) => UncheckedEvent::decode(&mut x.as_slice()).ok(),
			Err(_) => None,
		};

		Self {
			phase: value.phase,
			emitted_index: value.emitted_index,
			event,
		}
	}
}
