use crate::{
	AvailHeader, Client, Sub,
	block::{Block, BlockEvents, BlockEventsOptions},
};
use avail_rust_core::{
	BlockRef, RpcError,
	rpc::{BlockPhaseEvent, LegacyBlock},
};
use std::time::Duration;

/// Subscription that mirrors [`Sub`] but yields [`LegacyBlock`].
#[derive(Clone)]
pub struct LegacyBlockSub {
	sub: Sub,
}

impl LegacyBlockSub {
	/// Creates a new [`LegacyBlock`] subscription.
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	/// Returns the next [`LegacyBlock`] matching the underlying [`Sub::next`] cursor.
	///
	/// When the RPC call fails, the internal height is rewound so the same block can be retried.
	pub async fn next(&mut self) -> Result<Option<LegacyBlock>, RpcError> {
		let info = self.sub.next().await?;
		let block = match self
			.sub
			.client_ref()
			.rpc()
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

	/// Returns the previous [`LegacyBlock`] using [`Sub::prev`] as the cursor source.
	///
	/// When the RPC call fails, the internal height is rewound so the same block can be retried.
	pub async fn prev(&mut self) -> Result<Option<LegacyBlock>, RpcError> {
		let info = self.sub.prev().await?;
		let block = match self
			.sub
			.client_ref()
			.rpc()
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

	/// Delegates to [`Sub::use_best_block`].
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Delegates to [`Sub::set_block_height`].
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Delegates to [`Sub::set_pool_rate`].
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Delegates to [`Sub::set_retry_on_error`].
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

/// Subscription that mirrors [`Sub`] but yields [`Block`].
#[derive(Clone)]
pub struct BlockSub {
	sub: Sub,
}

impl BlockSub {
	/// Creates a new [`Block`] subscription.
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	/// Advances the subscription and returns a [`Block`] view alongside the originating [`BlockRef`].
	pub async fn next(&mut self) -> Result<(Block, BlockRef), RpcError> {
		let info = self.sub.next().await?;
		Ok((Block::new(self.sub.client_ref().clone(), info.hash), info))
	}

	/// Moves the subscription backwards and returns a [`Block`] view alongside the originating [`BlockRef`]..
	pub async fn prev(&mut self) -> Result<(Block, BlockRef), RpcError> {
		let info = self.sub.prev().await?;
		Ok((Block::new(self.sub.client_ref().clone(), info.hash), info))
	}

	/// Delegates to [`Sub::use_best_block`].
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Delegates to [`Sub::set_block_height`].
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Delegates to [`Sub::set_pool_rate`].
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Delegates to [`Sub::set_retry_on_error`].
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

/// Subscription that mirrors [`Sub`] but yields an array of[`BlockPhaseEvent`].
#[derive(Clone)]
pub struct BlockEventsSub {
	sub: Sub,
	opts: BlockEventsOptions,
}

impl BlockEventsSub {
	/// Creates a new [`BlockPhaseEvent`] subscription.
	pub fn new(client: Client, opts: BlockEventsOptions) -> Self {
		Self { sub: Sub::new(client), opts }
	}

	/// Advances to the next block that yields one or more events matching the configured options.
	///
	/// When the RPC call fails, the internal height is rewound so the same block can be retried.
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

	/// Delegates to [`Sub::use_best_block`].
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Delegates to [`Sub::set_block_height`].
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Delegates to [`Sub::set_pool_rate`].
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Delegates to [`Sub::set_retry_on_error`].
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
			.rpc()
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
			.rpc()
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

	/// Delegates to [`Sub::use_best_block`].
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Delegates to [`Sub::set_block_height`].
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Delegates to [`Sub::set_pool_rate`].
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Delegates to [`Sub::set_retry_on_error`].
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
		let (rpc_client, _commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical block
		let mut sub = BlockSub::new(client.clone());
		sub.set_block_height(1908729);

		let (_, info) = sub.next().await?;
		assert_eq!(info.height, 1908729);

		let (_, info) = sub.next().await?;
		assert_eq!(info.height, 1908730);

		let (_, info) = sub.prev().await?;
		assert_eq!(info.height, 1908729);

		let (_, info) = sub.prev().await?;
		assert_eq!(info.height, 1908728);

		// Best Block
		let expected = client.best().block_height().await?;
		let mut sub = BlockSub::new(client.clone());
		sub.use_best_block(true);

		let (_, info) = sub.next().await?;
		assert_eq!(info.height, expected);

		// Finalized Block
		let expected = client.finalized().block_height().await?;
		let mut sub = BlockSub::new(client);
		sub.use_best_block(false);

		let (_, info) = sub.next().await?;
		assert_eq!(info.height, expected);

		Ok(())
	}

	// This test will be by flaky and that is OK.
	#[tokio::test]
	pub async fn header_sub_test() -> Result<(), Error> {
		let (rpc_client, _commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical block
		let mut sub = BlockHeaderSub::new(client.clone());
		sub.set_block_height(1908729);

		let header = sub.next().await?.unwrap();
		assert_eq!(header.number, 1908729);

		let header = sub.next().await?.unwrap();
		assert_eq!(header.number, 1908730);

		let header = sub.prev().await?.unwrap();
		assert_eq!(header.number, 1908729);

		let header = sub.prev().await?.unwrap();
		assert_eq!(header.number, 1908728);

		// Best Block
		let expected = client.best().block_height().await?;
		let mut sub = BlockHeaderSub::new(client.clone());
		sub.use_best_block(true);

		let header = sub.next().await?.unwrap();
		assert_eq!(header.number, expected);

		// Finalized Block
		let expected = client.finalized().block_height().await?;
		let mut sub = BlockHeaderSub::new(client);
		sub.use_best_block(false);

		let header = sub.next().await?.unwrap();
		assert_eq!(header.number, expected);

		Ok(())
	}

	// This test will be by flaky and that is OK.
	#[tokio::test]
	pub async fn block_w_just_sub_test() -> Result<(), Error> {
		let (rpc_client, _commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical block
		let mut sub = LegacyBlockSub::new(client.clone());
		sub.set_block_height(1908729);

		let block = sub.next().await?.unwrap();
		assert_eq!(block.block.header.number, 1908729);

		let block = sub.next().await?.unwrap();
		assert_eq!(block.block.header.number, 1908730);

		let block = sub.prev().await?.unwrap();
		assert_eq!(block.block.header.number, 1908729);

		let block = sub.prev().await?.unwrap();
		assert_eq!(block.block.header.number, 1908728);

		// Best Block
		let expected = client.best().block_height().await?;
		let mut sub = LegacyBlockSub::new(client.clone());
		sub.use_best_block(true);

		let block = sub.next().await?.unwrap();
		assert_eq!(block.block.header.number, expected);

		// Finalized Block
		let expected = client.finalized().block_height().await?;
		let mut sub = LegacyBlockSub::new(client);
		sub.use_best_block(false);

		let block = sub.next().await?.unwrap();
		assert_eq!(block.block.header.number, expected);

		Ok(())
	}
}
