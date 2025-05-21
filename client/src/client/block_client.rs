use crate::client::Client;
use core::{rpc, HashIndex, H256};

#[derive(Clone)]
pub struct BlockClient {
	client: Client,
}

impl BlockClient {
	pub fn new(client: Client) -> Self {
		Self { client }
	}

	pub async fn block(&self, at: H256) -> Result<Option<rpc::Block>, core::Error> {
		self.client.block(at).await.map(|b| b.map(|x| x.block))
	}

	pub async fn best_block(&self) -> Result<rpc::Block, core::Error> {
		self.client.best_block().await.map(|b| b.block)
	}

	pub async fn finalized_block(&self) -> Result<rpc::Block, core::Error> {
		self.client
			.finalized_block()
			.await
			.map(|b: rpc::BlockWithJustifications| b.block)
	}

	pub async fn block_data_builder(block_index: HashIndex) -> block_data::BlockBuilder {
		block_data::BlockBuilder::new(block_index)
	}

	pub async fn block_overview_builder(block_index: HashIndex) -> block_overview::BlockBuilder {
		block_overview::BlockBuilder::new(block_index)
	}
}

pub mod block_data {
	use crate::{client::Client, codec::Decode};
	use core::{
		config::TransactionLocation,
		decoded_transaction::{DecodedEvent, RuntimePhase},
		rpc::block::block_data as rpc_block_data,
		BlockId, DecodedTransaction, DispatchIndex, EmittedIndex, HashIndex,
	};

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

		pub async fn build(&self, client: &Client) -> Result<Block, core::Error> {
			let response = client.rpc_block_data(self.params.clone()).await.map(|x| x.value)?;
			let calls = response
				.calls
				.map(|list| list.into_iter().map(CallData::from).collect());

			let events = response
				.events
				.map(|list| list.into_iter().map(EventData::from).collect());

			Ok(Block {
				block_id: response.block_id,
				block_state: response.block_state,
				calls,
				events,
			})
		}
	}

	#[derive(Clone)]
	pub struct Block {
		pub block_id: BlockId,
		pub block_state: rpc_block_data::BlockState,
		pub calls: Option<Vec<CallData>>,
		pub events: Option<Vec<EventData>>,
	}

	#[derive(Clone)]
	pub struct CallData {
		pub tx_location: TransactionLocation,
		pub dispatch_index: DispatchIndex,
		// None if we failed to decode it
		pub call: Option<DecodedTransaction>,
	}

	impl From<rpc_block_data::CallData> for CallData {
		fn from(value: rpc_block_data::CallData) -> Self {
			let call = match hex::decode(value.call.trim_start_matches("0x")) {
				Ok(x) => DecodedTransaction::decode(&mut x.as_slice()).ok(),
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
		pub event: Option<DecodedEvent>,
	}

	impl From<rpc_block_data::EventData> for EventData {
		fn from(value: rpc_block_data::EventData) -> Self {
			let event = match hex::decode(value.event.trim_start_matches("0x")) {
				Ok(x) => DecodedEvent::decode(&mut x.as_slice()).ok(),
				Err(_) => None,
			};

			Self {
				phase: value.phase,
				emitted_index: value.emitted_index,
				event,
			}
		}
	}
}

pub mod block_overview {
	use crate::client::Client;
	use core::{rpc::block::block_overview as rpc_block_overview, HashIndex};

	#[derive(Clone)]
	pub struct BlockBuilder {
		params: rpc_block_overview::Params,
	}

	impl BlockBuilder {
		pub fn new(block_index: HashIndex) -> Self {
			Self {
				params: rpc_block_overview::Params::new(block_index),
			}
		}

		pub fn block_index(mut self, value: HashIndex) -> Self {
			self.params.block_index = value;
			self
		}

		pub fn enable_call_decoding(mut self, value: bool) -> Self {
			self.params.extension.enable_call_decoding = value;
			self
		}

		pub fn enable_event_decoding(mut self, value: bool) -> Self {
			self.params.extension.enable_event_decoding = value;
			self
		}

		pub fn enable_consensus_event(mut self, value: bool) -> Self {
			self.params.extension.enable_consensus_event = value;
			self
		}

		pub fn fetch_events(mut self, value: bool) -> Self {
			self.params.extension.fetch_events = value;
			self
		}

		pub fn transaction_filter(mut self, value: rpc_block_overview::TransactionFilterOptions) -> Self {
			self.params.filter.transaction = value;
			self
		}

		pub fn signature_filter(mut self, value: rpc_block_overview::SignatureFilterOptions) -> Self {
			self.params.filter.signature = value;
			self
		}

		pub async fn build(&self, client: &Client) -> Result<rpc_block_overview::Block, core::Error> {
			client.rpc_block_overview(self.params.clone()).await.map(|x| x.value)
		}
	}
}
