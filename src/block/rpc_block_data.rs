use crate::{client::rpc::rpc_block_data, client::Client, config::HashIndex};

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

	pub async fn build(&self, client: &Client) -> Result<rpc_block_data::Block, subxt_rpcs::Error> {
		client.rpc_block_data(self.params.clone()).await.map(|x| x.value)
	}
}
