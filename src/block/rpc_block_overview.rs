use crate::client::Client;
use crate::error::RpcError;
use crate::primitives::rpc::block::block_overview;
use crate::primitives::HashIndex;

#[derive(Clone)]
pub struct BlockBuilder {
	params: block_overview::Params,
}

impl BlockBuilder {
	pub fn new(block_index: HashIndex) -> Self {
		Self {
			params: block_overview::Params::new(block_index),
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

	pub fn transaction_filter(mut self, value: block_overview::TransactionFilterOptions) -> Self {
		self.params.filter.transaction = value;
		self
	}

	pub fn signature_filter(mut self, value: block_overview::SignatureFilterOptions) -> Self {
		self.params.filter.signature = value;
		self
	}

	pub async fn build(&self, client: &Client) -> Result<block_overview::Block, RpcError> {
		client.rpc_block_overview(self.params.clone()).await.map(|x| x.value)
	}
}
