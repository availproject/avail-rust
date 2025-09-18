use crate::{
	AvailHeader, Client,
	block::{
		BlockEvents, BlockEventsOptions, BlockExtOptionsExpanded, BlockExtOptionsSimple, BlockExtrinsic,
		BlockRawExtrinsic, BlockSignedExtrinsic, BlockWithExt, BlockWithRawExt, BlockWithTx,
	},
	platform::sleep,
};
use avail_rust_core::{
	BlockRef, H256, HasHeader, RpcError,
	grandpa::GrandpaJustification,
	rpc::{BlockPhaseEvent, BlockWithJustifications},
};
use codec::Decode;
use std::{marker::PhantomData, time::Duration};

#[derive(Clone)]
pub struct SubBuilder {
	use_best_block: bool,
	block_height: Option<u32>,
	poll_rate: Duration,
	retry_on_error: bool,
}

impl SubBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn follow(mut self, best_block: bool) -> Self {
		self.use_best_block = best_block;
		self
	}

	pub fn block_height(mut self, value: u32) -> Self {
		self.block_height = Some(value);
		self
	}

	pub fn poll_rate(mut self, value: Duration) -> Self {
		self.poll_rate = value;
		self
	}

	pub fn retry_on_error(mut self, value: bool) -> Self {
		self.retry_on_error = value;
		self
	}

	pub async fn build(&self, client: &Client) -> Result<Sub, RpcError> {
		let block_height = match self.block_height {
			Some(x) => x,
			None => match self.use_best_block {
				true => client.best().block_height().await?,
				false => client.finalized().block_height().await?,
			},
		};

		let sub = match self.use_best_block {
			true => Sub::BestBlock(BestBlockSub {
				poll_rate: self.poll_rate,
				current_block_height: block_height,
				block_processed: Vec::new(),
				retry_on_error: self.retry_on_error,
				latest_finalized_height: None,
			}),
			false => Sub::FinalizedBlock(FinalizedBlockSub {
				poll_rate: self.poll_rate,
				next_block_height: block_height,
				retry_on_error: self.retry_on_error,
				latest_finalized_height: None,
			}),
		};
		Ok(sub)
	}
}

impl Default for SubBuilder {
	fn default() -> Self {
		Self {
			use_best_block: false,
			block_height: Default::default(),
			poll_rate: Duration::from_secs(3),
			retry_on_error: true,
		}
	}
}

#[derive(Clone)]
pub struct FinalizedBlockSub {
	poll_rate: Duration,
	next_block_height: u32,
	retry_on_error: bool,
	latest_finalized_height: Option<u32>,
}

impl FinalizedBlockSub {
	pub async fn run(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		let latest_finalized_height = self.fetch_latest_finalized_height(client).await?;

		let result = if latest_finalized_height >= self.next_block_height {
			self.run_historical(client).await?
		} else {
			self.run_head(client).await?
		};

		self.next_block_height = result.height + 1;
		Ok(result)
	}

	pub fn current_block_height(&self) -> u32 {
		self.next_block_height.saturating_sub(1)
	}

	async fn fetch_latest_finalized_height(&mut self, client: &Client) -> Result<u32, RpcError> {
		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		let latest_finalized_height = client.finalized().block_height().await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	async fn run_historical(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		let height = self.next_block_height;
		let hash = client
			.rpc()
			.retry_on(Some(self.retry_on_error), None)
			.block_hash(Some(height))
			.await?;
		let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

		Ok(BlockRef { hash, height })
	}

	async fn run_head(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		loop {
			let head = client.finalized().block_info().await?;

			let is_past_block = self.next_block_height > head.height;
			if is_past_block {
				sleep(self.poll_rate).await;
				continue;
			}

			if self.next_block_height == head.height {
				return Ok(head);
			}

			let height = self.next_block_height;
			let hash = client
				.rpc()
				.retry_on(Some(self.retry_on_error), Some(true))
				.block_hash(Some(height))
				.await?;
			let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

			return Ok(BlockRef { hash, height });
		}
	}
}

#[derive(Clone)]
pub struct BestBlockSub {
	poll_rate: Duration,
	current_block_height: u32,
	block_processed: Vec<H256>,
	retry_on_error: bool,
	latest_finalized_height: Option<u32>,
}

impl BestBlockSub {
	pub async fn run(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		let latest_finalized_height = self.fetch_latest_finalized_height(client).await?;

		// Dealing with historical blocks
		if latest_finalized_height >= self.current_block_height {
			let info = self.run_historical(client).await?;
			self.current_block_height = info.height + 1;
			return Ok(info);
		}

		let info = self.run_head(client).await?;
		if info.height == self.current_block_height {
			self.block_processed.push(info.hash);
		} else {
			self.block_processed = vec![info.hash];
			self.current_block_height = info.height;
		}

		Ok(info)
	}

	pub fn current_block_height(&self) -> u32 {
		self.current_block_height
	}

	async fn fetch_latest_finalized_height(&mut self, client: &Client) -> Result<u32, RpcError> {
		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		let latest_finalized_height = client.finalized().block_height().await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	async fn run_historical(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		let height = self.current_block_height;
		let hash = client
			.rpc()
			.retry_on(Some(self.retry_on_error), None)
			.block_hash(Some(height))
			.await?;
		let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

		Ok(BlockRef { hash, height })
	}

	async fn run_head(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		loop {
			let head = client.best().block_info().await?;

			let is_past_block = self.current_block_height > head.height;
			let block_already_processed = self.block_processed.contains(&head.hash);
			if is_past_block || block_already_processed {
				sleep(self.poll_rate).await;
				continue;
			}

			let is_current_block = self.current_block_height == head.height;
			let is_next_block = self.current_block_height + 1 == head.height;
			if is_current_block || is_next_block {
				return Ok(head);
			}

			let no_block_processed_yet = self.block_processed.is_empty();
			if no_block_processed_yet {
				let hash = client
					.rpc()
					.retry_on(Some(true), Some(true))
					.block_hash(Some(self.current_block_height))
					.await?;
				let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

				return Ok(BlockRef { hash, height: self.current_block_height });
			}

			let height = self.current_block_height + 1;
			let hash = client
				.rpc()
				.retry_on(Some(true), Some(true))
				.block_hash(Some(height))
				.await?;
			let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

			return Ok(BlockRef { hash, height });
		}
	}
}

#[derive(Clone)]
pub enum Sub {
	BestBlock(BestBlockSub),
	FinalizedBlock(FinalizedBlockSub),
}

impl Sub {
	pub async fn next(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		match self {
			Self::BestBlock(s) => s.run(client).await,
			Self::FinalizedBlock(s) => s.run(client).await,
		}
	}

	pub fn retry_on_error(&self) -> bool {
		match self {
			Self::BestBlock(s) => s.retry_on_error,
			Self::FinalizedBlock(s) => s.retry_on_error,
		}
	}

	pub fn current_block_height(&self) -> u32 {
		match self {
			Self::BestBlock(s) => s.current_block_height(),
			Self::FinalizedBlock(s) => s.current_block_height(),
		}
	}

	pub fn revert_to(&mut self, block_height: u32) {
		match self {
			Sub::BestBlock(x) => {
				x.block_processed.clear();
				x.current_block_height = block_height;
			},
			Sub::FinalizedBlock(x) => x.next_block_height = block_height,
		}
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		match self {
			Sub::BestBlock(x) => x.poll_rate = value,
			Sub::FinalizedBlock(x) => x.poll_rate = value,
		}
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		match self {
			Sub::BestBlock(x) => x.retry_on_error = value,
			Sub::FinalizedBlock(x) => x.retry_on_error = value,
		}
	}
}

#[derive(Clone)]
pub struct BlockWithJustSub {
	client: Client,
	sub: Sub,
	retry_on_error: bool,
}

impl BlockWithJustSub {
	pub fn new(client: Client, sub: Sub) -> Self {
		let retry_on_error = sub.retry_on_error();
		Self { client, sub, retry_on_error }
	}

	pub async fn next(&mut self) -> Result<Option<BlockWithJustifications>, RpcError> {
		let info = self.sub.next(&self.client).await?;
		let block = match self
			.client
			.rpc()
			.retry_on(Some(self.retry_on_error), Some(true))
			.block_with_justification(Some(info.hash))
			.await
		{
			Ok(x) => x,
			Err(err) => {
				// Revet block height if we fail to fetch block
				self.sub.revert_to(info.height);
				return Err(err);
			},
		};
		Ok(block)
	}
}

#[derive(Clone)]
pub struct BlockSub {
	client: Client,
	sub: Sub,
}

impl BlockSub {
	pub fn new(client: Client, sub: Sub) -> Self {
		Self { client, sub }
	}

	pub async fn next(&mut self) -> Result<crate::block::Block, RpcError> {
		let info = self.sub.next(&self.client).await?;
		Ok(crate::block::Block::new(self.client.clone(), info.hash))
	}

	pub fn revert_to(&mut self, block_height: u32) {
		self.sub.revert_to(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct TransactionSub<T: HasHeader + Decode> {
	client: Client,
	sub: Sub,
	opts: BlockExtOptionsSimple,
	_phantom: PhantomData<T>,
}

impl<T: HasHeader + Decode> TransactionSub<T> {
	pub fn new(client: Client, sub: Sub, opts: BlockExtOptionsSimple) -> Self {
		Self { client, sub, opts, _phantom: Default::default() }
	}

	pub async fn next(&mut self) -> Result<(Vec<BlockSignedExtrinsic<T>>, BlockRef), crate::Error> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let block = BlockWithTx::new(self.client.clone(), info.hash);

			let txs = match block.all::<T>(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.revert_to(info.height);
					return Err(err);
				},
			};

			if txs.is_empty() {
				continue;
			}

			return Ok((txs, info));
		}
	}

	pub fn revert_to(&mut self, block_height: u32) {
		self.sub.revert_to(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct ExtrinsicSub<T: HasHeader + Decode> {
	client: Client,
	sub: Sub,
	opts: BlockExtOptionsSimple,
	_phantom: PhantomData<T>,
}

impl<T: HasHeader + Decode> ExtrinsicSub<T> {
	pub fn new(client: Client, sub: Sub, opts: BlockExtOptionsSimple) -> Self {
		Self { client, sub, opts, _phantom: Default::default() }
	}

	pub async fn next(&mut self) -> Result<(Vec<BlockExtrinsic<T>>, BlockRef), crate::Error> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let block = BlockWithExt::new(self.client.clone(), info.hash);

			let txs = match block.all::<T>(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.revert_to(info.height);
					return Err(err);
				},
			};

			if txs.is_empty() {
				continue;
			}

			return Ok((txs, info));
		}
	}

	pub fn revert_to(&mut self, block_height: u32) {
		self.sub.revert_to(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct RawExtrinsicSub {
	client: Client,
	sub: Sub,
	opts: BlockExtOptionsExpanded,
}

impl RawExtrinsicSub {
	pub fn new(client: Client, sub: Sub, opts: BlockExtOptionsExpanded) -> Self {
		Self { client, sub, opts }
	}

	pub async fn next(&mut self) -> Result<(Vec<BlockRawExtrinsic>, BlockRef), crate::Error> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let block = BlockWithRawExt::new(self.client.clone(), info.hash);

			let txs = match block.all(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.revert_to(info.height);
					return Err(err);
				},
			};

			if txs.is_empty() {
				continue;
			}

			return Ok((txs, info));
		}
	}

	pub fn revert_to(&mut self, block_height: u32) {
		self.sub.revert_to(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct EventsSub {
	client: Client,
	sub: Sub,
	opts: BlockEventsOptions,
}

impl EventsSub {
	pub fn new(client: Client, sub: Sub, opts: BlockEventsOptions) -> Self {
		Self { client, sub, opts }
	}

	pub async fn next(&mut self) -> Result<Vec<BlockPhaseEvent>, crate::Error> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let block = BlockEvents::new(self.client.clone(), info.hash);
			let events = match block.block(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch events
					self.sub.revert_to(info.height);
					return Err(err);
				},
			};

			if events.is_empty() {
				continue;
			}

			return Ok(events);
		}
	}

	pub fn revert_to(&mut self, block_height: u32) {
		self.sub.revert_to(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct BlockHeaderSub {
	client: Client,
	sub: Sub,
	retry_on_error: bool,
}

impl BlockHeaderSub {
	pub fn new(client: Client, sub: Sub) -> Self {
		let retry_on_error = sub.retry_on_error();
		Self { client, sub, retry_on_error }
	}

	pub async fn next(&mut self) -> Result<Option<AvailHeader>, RpcError> {
		let info = self.sub.next(&self.client).await?;
		let header = match self
			.client
			.rpc()
			.retry_on(Some(self.retry_on_error), Some(true))
			.block_header(Some(info.hash))
			.await
		{
			Ok(x) => x,
			Err(err) => {
				// Revet block height if we fail to fetch block header
				self.sub.revert_to(info.height);
				return Err(err);
			},
		};

		Ok(header)
	}

	pub fn revert_to(&mut self, block_height: u32) {
		self.sub.revert_to(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct GrandpaJustificationSub {
	client: Client,
	sub: Sub,
}

impl GrandpaJustificationSub {
	pub fn new(client: Client, next_block_height: u32) -> Self {
		let sub = Sub::FinalizedBlock(FinalizedBlockSub {
			poll_rate: Duration::from_secs(3),
			next_block_height,
			retry_on_error: true,
			latest_finalized_height: None,
		});

		Self { client, sub }
	}

	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let just = match self.client.rpc().grandpa_block_justification(info.height).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.revert_to(info.height);
					return Err(err);
				},
			};

			let Some(just) = just else {
				continue;
			};

			return Ok(just);
		}
	}

	pub fn revert_to(&mut self, block_height: u32) {
		self.sub.revert_to(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct GrandpaJustificationJsonSub {
	client: Client,
	sub: Sub,
}

impl GrandpaJustificationJsonSub {
	pub fn new(client: Client, next_block_height: u32) -> Self {
		let sub = Sub::FinalizedBlock(FinalizedBlockSub {
			poll_rate: Duration::from_secs(3),
			next_block_height,
			retry_on_error: true,
			latest_finalized_height: None,
		});

		Self { client, sub }
	}

	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let just = match self.client.rpc().grandpa_block_justification_json(info.height).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.revert_to(info.height);
					return Err(err);
				},
			};

			let Some(just) = just else {
				continue;
			};

			return Ok(just);
		}
	}

	pub fn revert_to(&mut self, block_height: u32) {
		self.sub.revert_to(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}
