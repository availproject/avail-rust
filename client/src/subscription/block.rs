use crate::{
	AvailHeader, Client, Sub,
	block::{Block, BlockEvents, BlockEventsOptions},
};
use avail_rust_core::{
	BlockRef, RpcError,
	rpc::{BlockPhaseEvent, LegacyBlock},
};
use std::time::Duration;

/// The `BlockWithJustSub` subscription behaves just as [Sub]
///
/// The difference is that instead of fetching block (hash, height) it
/// fetches full blocks with justification [LegacyBlock].
#[derive(Clone)]
pub struct LegacyBlockSub {
	sub: Sub,
}

impl LegacyBlockSub {
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	pub async fn next(&mut self) -> Result<Option<LegacyBlock>, RpcError> {
		let info = self.sub.next().await?;
		let block = match self
			.sub
			.client_ref()
			.rpc()
			.retry_on(Some(self.sub.should_retry_on_error()), Some(true))
			.block_with_justification(Some(info.hash))
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

	pub async fn prev(&mut self) -> Result<Option<LegacyBlock>, RpcError> {
		let info = self.sub.prev().await?;
		let block = match self
			.sub
			.client_ref()
			.rpc()
			.retry_on(Some(self.sub.should_retry_on_error()), Some(true))
			.block_with_justification(Some(info.hash))
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

	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

/// The `BlockSub` subscription behaves just as [Sub]
///
/// The difference is that instead of fetching block (hash, height) it
/// constructs an instance of [Block]
#[derive(Clone)]
pub struct BlockSub {
	sub: Sub,
}

impl BlockSub {
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	pub async fn next(&mut self) -> Result<(Block, BlockRef), RpcError> {
		let info = self.sub.next().await?;
		Ok((Block::new(self.sub.client_ref().clone(), info.hash), info))
	}

	pub async fn prev(&mut self) -> Result<(Block, BlockRef), RpcError> {
		let info = self.sub.prev().await?;
		Ok((Block::new(self.sub.client_ref().clone(), info.hash), info))
	}

	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct EventsSub {
	sub: Sub,
	opts: BlockEventsOptions,
}

impl EventsSub {
	pub fn new(client: Client, opts: BlockEventsOptions) -> Self {
		Self { sub: Sub::new(client), opts }
	}

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

	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

/// The `BlockHeaderSub` subscription behaves just as [Sub]
///
/// The difference is that instead of fetching block (hash, height) it
/// fetches block headers [AvailHeader]
#[derive(Clone)]
pub struct BlockHeaderSub {
	sub: Sub,
}

impl BlockHeaderSub {
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

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

	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

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
