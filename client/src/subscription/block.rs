use crate::{
	AvailHeader, BlockInfo, Client, LegacyBlock, RpcError, Sub,
	block::{BlockApi, BlockEvents, BlockEventsOptions},
};
use avail_rust_core::rpc::BlockPhaseEvent;
use std::time::Duration;

/// Subscription wrapper that streams [`LegacyBlock`] values.
#[derive(Clone)]
pub struct LegacyBlockSub {
	sub: Sub,
}

impl LegacyBlockSub {
	/// Creates a subscription that yields legacy blocks as you iterate.
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	/// Fetches the next legacy block; rewinds the cursor if the RPC call fails.
	pub async fn next(&mut self) -> Result<Option<LegacyBlock>, RpcError> {
		let info = self.sub.next().await?;
		let block = match self
			.sub
			.client_ref()
			.chain()
			.retry_on(Some(self.sub.should_retry_on_error()), Some(true))
			.legacy_block(Some(info.hash))
			.await
		{
			Ok(x) => x,
			Err(err) => {
				// Revet block height if we fail to fetch block
				self.sub.set_block_height(info.height);
				return Err(err);
			},
		};
		Ok(block)
	}

	/// Fetches the previous legacy block; rewinds the cursor if the RPC call fails.
	pub async fn prev(&mut self) -> Result<Option<LegacyBlock>, RpcError> {
		let info = self.sub.prev().await?;
		let block = match self
			.sub
			.client_ref()
			.chain()
			.retry_on(Some(self.sub.should_retry_on_error()), Some(true))
			.legacy_block(Some(info.hash))
			.await
		{
			Ok(x) => x,
			Err(err) => {
				// Revet block height if we fail to fetch block
				self.sub.set_block_height(info.height);
				return Err(err);
			},
		};
		Ok(block)
	}

	/// Reports whether failed RPC calls will be retried.
	pub fn should_retry_on_error(&self) -> bool {
		self.sub.should_retry_on_error()
	}

	/// Follow best blocks instead of finalized ones.
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Jump the cursor to a specific starting height.
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Change how often we poll for new blocks.
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Controls retry behaviour: `Some(true)` forces retries, `Some(false)` disables them, and `None`
	/// keeps the client's default.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

/// Subscription wrapper that streams [`Block`] handles.
#[derive(Clone)]
pub struct BlockSub {
	sub: Sub,
}

impl BlockSub {
	/// Creates a subscription that yields [`Block`] handles as you iterate.
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	/// Fetches the next block handle along with its `BlockInfo`.
	pub async fn next(&mut self) -> Result<(BlockApi, BlockInfo), RpcError> {
		let info = self.sub.next().await?;
		Ok((BlockApi::new(self.sub.client_ref().clone(), info.hash), info))
	}

	/// Fetches the previous block handle along with its `BlockInfo`.
	pub async fn prev(&mut self) -> Result<(BlockApi, BlockInfo), RpcError> {
		let info = self.sub.prev().await?;
		Ok((BlockApi::new(self.sub.client_ref().clone(), info.hash), info))
	}

	/// Reports whether failed RPC calls will be retried.
	pub fn should_retry_on_error(&self) -> bool {
		self.sub.should_retry_on_error()
	}

	/// Follow best blocks instead of finalized ones.
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Jump the cursor to a specific starting height.
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Change how often we poll for new blocks.
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Controls retry behaviour: `Some(true)` forces retries, `Some(false)` disables them, and `None`
	/// keeps the client's default.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

/// Subscription wrapper that streams [`BlockPhaseEvent`] lists.
#[derive(Clone)]
pub struct BlockEventsSub {
	sub: Sub,
	opts: BlockEventsOptions,
}

impl BlockEventsSub {
	/// Creates a subscription that yields event batches filtered by the supplied options.
	pub fn new(client: Client, opts: BlockEventsOptions) -> Self {
		Self { sub: Sub::new(client), opts }
	}

	/// Fetches the next block with matching events; rewinds on RPC failure.
	pub async fn next(&mut self) -> Result<Vec<BlockPhaseEvent>, crate::Error> {
		loop {
			let info = self.sub.next().await?;
			let block = BlockEvents::new(self.sub.client_ref().clone(), info.hash);
			let events = match block.block(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch events
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			if events.is_empty() {
				continue;
			}

			return Ok(events);
		}
	}

	/// Reports whether failed RPC calls will be retried.
	pub fn should_retry_on_error(&self) -> bool {
		self.sub.should_retry_on_error()
	}

	/// Follow best blocks instead of finalized ones.
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Jump the cursor to a specific starting height.
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Change how often we poll for new blocks.
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Override the retry behaviour (`Some(true)` = force, `Some(false)` = disable).
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

/// Subscription that mirrors [`Sub`] but yields [`AvailHeader`].
#[derive(Clone)]
pub struct BlockHeaderSub {
	sub: Sub,
}

impl BlockHeaderSub {
	/// Creates a new [`AvailHeader`] subscription.
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	/// Returns the next [`AvailHeader`] matching the underlying [`Sub::next`] cursor.
	///
	/// When the RPC call fails, the internal height is rewound so the same block can be retried.
	pub async fn next(&mut self) -> Result<Option<AvailHeader>, crate::Error> {
		let info = self.sub.next().await?;
		let header = match self
			.sub
			.client_ref()
			.chain()
			.retry_on(Some(self.sub.should_retry_on_error()), Some(true))
			.block_header(Some(info.hash))
			.await
		{
			Ok(x) => x,
			Err(err) => {
				// Revet block height if we fail to fetch block header
				self.sub.set_block_height(info.height);
				return Err(err);
			},
		};

		Ok(header)
	}

	/// Returns the previous [`AvailHeader`] using [`Sub::prev`] as the cursor source.
	///
	/// When the RPC call fails, the internal height is rewound so the same block can be retried.
	pub async fn prev(&mut self) -> Result<Option<AvailHeader>, crate::Error> {
		let info = self.sub.prev().await?;
		let header = match self
			.sub
			.client_ref()
			.chain()
			.retry_on(Some(self.sub.should_retry_on_error()), Some(true))
			.block_header(Some(info.hash))
			.await
		{
			Ok(x) => x,
			Err(err) => {
				// Revet block height if we fail to fetch block header
				self.sub.set_block_height(info.height);
				return Err(err);
			},
		};

		Ok(header)
	}

	pub fn should_retry_on_error(&self) -> bool {
		self.sub.should_retry_on_error()
	}

	/// Follow best blocks instead of finalized ones.
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Jump the cursor to a specific starting height.
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Change how often we poll for new blocks.
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Choose whether this subscription should retry after RPC failures.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{clients::mock_client::MockClient, error::Error, prelude::*, subxt_rpcs::RpcClient};

	// This test will be by flaky and that is OK.
	#[tokio::test]
	pub async fn block_sub_test() -> Result<(), Error> {
		let client = Client::new(TURING_ENDPOINT).await?;

		//
		// Test Case 1: Latest Block Height + Next
		//
		let mut sub = BlockSub::new(client.clone());

		let block_height = client.finalized().block_height().await?;
		let value = sub.next().await?;
		assert_eq!(value.1.height, block_height);

		//
		// Test Case 2: Latest Block Height + Prev
		//
		let mut sub = BlockSub::new(client.clone());

		let block_height = client.finalized().block_height().await?;
		let value = sub.prev().await?;
		assert_eq!(value.1.height, block_height - 1);

		//
		// Test Case 3: Set Block Height + Next + Next + Next
		//
		let block_height = 1900000u32;
		let mut sub = BlockSub::new(client.clone());
		sub.set_block_height(block_height);
		for i in 0..3 {
			let value = sub.next().await?;
			assert_eq!(value.1.height, block_height + i);
		}

		//
		// Test Case 4: Set Block Height + Prev + Prev + Prev
		//
		let block_height = 1900000u32;
		let mut sub = BlockSub::new(client.clone());
		sub.set_block_height(block_height);
		for i in 0..3 {
			let value = sub.prev().await?;
			assert_eq!(value.1.height, block_height - i - 1);
		}

		//
		// Test Case 5: Set Block Height + Next + Prev
		//
		let block_height = 1900000u32;
		let mut sub = BlockSub::new(client.clone());
		sub.set_block_height(block_height);

		let value = sub.next().await?;
		assert_eq!(value.1.height, block_height);

		let value = sub.prev().await?;
		assert_eq!(value.1.height, block_height - 1);

		//
		// Test Case 6: Set Block Height + Prev + Next
		//
		let block_height = 1900000u32;
		let mut sub = BlockSub::new(client.clone());
		sub.set_block_height(block_height);

		let value = sub.prev().await?;
		assert_eq!(value.1.height, block_height - 1);

		let value = sub.next().await?;
		assert_eq!(value.1.height, block_height);

		Ok(())
	}

	// This test will be by flaky and that is OK.
	#[tokio::test]
	pub async fn header_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		//
		// Test Case 1: Latest Block Height + Next
		//
		let mut sub = BlockHeaderSub::new(client.clone());

		let block_height = client.finalized().block_height().await?;
		let value = sub.next().await?.expect("Should be there");
		assert_eq!(value.number, block_height);

		//
		// Test Case 2: Latest Block Height + Prev
		//
		let mut sub = BlockHeaderSub::new(client.clone());

		let block_height = client.finalized().block_height().await?;
		let value = sub.prev().await?.expect("Should be there");
		assert_eq!(value.number, block_height - 1);

		//
		// Test Case 3: Set Block Height + Next + Next + Next
		//
		let block_height = 1900000u32;
		let mut sub = BlockHeaderSub::new(client.clone());
		sub.set_block_height(block_height);
		for i in 0..3 {
			let value = sub.next().await?.expect("Should be there");
			assert_eq!(value.number, block_height + i);
		}

		//
		// Test Case 4: Set Block Height + Prev + Prev + Prev
		//
		let block_height = 1900000u32;
		let mut sub = BlockHeaderSub::new(client.clone());
		sub.set_block_height(block_height);
		for i in 0..3 {
			let value = sub.prev().await?.expect("Should be there");
			assert_eq!(value.number, block_height - i - 1);
		}

		//
		// Test Case 5: Set Block Height + Next + Prev
		//
		let block_height = 1900000u32;
		let mut sub = BlockHeaderSub::new(client.clone());
		sub.set_block_height(block_height);

		let value = sub.next().await?.expect("Should be there");
		assert_eq!(value.number, block_height);

		let value = sub.prev().await?.expect("Should be there");
		assert_eq!(value.number, block_height - 1);

		//
		// Test Case 6: Set Block Height + Prev + Next
		//
		let block_height = 1900000u32;
		let mut sub = BlockHeaderSub::new(client.clone());
		sub.set_block_height(block_height);

		let value = sub.prev().await?.expect("Should be there");
		assert_eq!(value.number, block_height - 1);

		let value = sub.next().await?.expect("Should be there");
		assert_eq!(value.number, block_height);

		//
		// Test Case 6: Set Block Height + Next + Fail + Next
		//
		let block_height = 1900000u32;
		let mut sub = BlockHeaderSub::new(client.clone());
		sub.set_retry_on_error(Some(false));
		sub.set_block_height(block_height);

		let value = sub.next().await?.expect("Should be there");
		assert_eq!(value.number, block_height);
		assert_eq!(sub.sub.as_finalized().next_block_height, block_height + 1);

		commander.block_header_err(None);
		let _ = sub.next().await.expect_err("Should fail");
		assert_eq!(sub.sub.as_finalized().next_block_height, block_height + 1);

		let value = sub.next().await?.expect("Should be there");
		assert_eq!(value.number, block_height + 1);
		assert_eq!(sub.sub.as_finalized().next_block_height, block_height + 2);

		Ok(())
	}

	// This test will be by flaky and that is OK.
	#[tokio::test]
	pub async fn legacy_block_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		//
		// Test Case 1: Latest Block Height + Next
		//
		let mut sub = LegacyBlockSub::new(client.clone());

		let block_height = client.finalized().block_height().await?;
		let value = sub.next().await?.expect("Should be there");
		assert_eq!(value.block.header.number, block_height);

		//
		// Test Case 2: Latest Block Height + Prev
		//
		let mut sub = LegacyBlockSub::new(client.clone());

		let block_height = client.finalized().block_height().await?;
		let value = sub.prev().await?.expect("Should be there");
		assert_eq!(value.block.header.number, block_height - 1);

		//
		// Test Case 3: Set Block Height + Next + Next + Next
		//
		let block_height = 1900000u32;
		let mut sub = LegacyBlockSub::new(client.clone());
		sub.set_block_height(block_height);
		for i in 0..3 {
			let value = sub.next().await?.expect("Should be there");
			assert_eq!(value.block.header.number, block_height + i);
		}

		//
		// Test Case 4: Set Block Height + Prev + Prev + Prev
		//
		let block_height = 1900000u32;
		let mut sub = LegacyBlockSub::new(client.clone());
		sub.set_block_height(block_height);
		for i in 0..3 {
			let value = sub.prev().await?.expect("Should be there");
			assert_eq!(value.block.header.number, block_height - i - 1);
		}

		//
		// Test Case 5: Set Block Height + Next + Prev
		//
		let block_height = 1900000u32;
		let mut sub = LegacyBlockSub::new(client.clone());
		sub.set_block_height(block_height);

		let value = sub.next().await?.expect("Should be there");
		assert_eq!(value.block.header.number, block_height);

		let value = sub.prev().await?.expect("Should be there");
		assert_eq!(value.block.header.number, block_height - 1);

		//
		// Test Case 6: Set Block Height + Prev + Next
		//
		let block_height = 1900000u32;
		let mut sub = LegacyBlockSub::new(client.clone());
		sub.set_block_height(block_height);

		let value = sub.prev().await?.expect("Should be there");
		assert_eq!(value.block.header.number, block_height - 1);

		let value = sub.next().await?.expect("Should be there");
		assert_eq!(value.block.header.number, block_height);

		//
		// Test Case 6: Set Block Height + Next + Fail + Next
		//
		let block_height = 1900000u32;
		let mut sub = LegacyBlockSub::new(client.clone());
		sub.set_retry_on_error(Some(false));
		sub.set_block_height(block_height);

		let value = sub.next().await?.expect("Should be there");
		assert_eq!(value.block.header.number, block_height);
		assert_eq!(sub.sub.as_finalized().next_block_height, block_height + 1);

		commander.legacy_block_err(None);
		let _ = sub.next().await.expect_err("Should fail");
		assert_eq!(sub.sub.as_finalized().next_block_height, block_height + 1);

		let value = sub.next().await?.expect("Should be there");
		assert_eq!(value.block.header.number, block_height + 1);
		assert_eq!(sub.sub.as_finalized().next_block_height, block_height + 2);

		Ok(())
	}
}
