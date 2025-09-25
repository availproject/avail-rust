use crate::{
	AvailHeader, Client,
	block::{
		Block, BlockEvents, BlockEventsOptions, BlockExtOptionsExpanded, BlockExtOptionsSimple, BlockExtrinsic,
		BlockRawExtrinsic, BlockTransaction, BlockWithExt, BlockWithRawExt, BlockWithTx,
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

/// Dummy subscription. Not meant to be used directly.
///
/// Use [Sub] instead.
#[derive(Clone)]
pub struct UnInitSub {
	client: Client,
	use_best_block: bool,
	block_height: Option<u32>,
	poll_rate: Duration,
	retry_on_error: Option<bool>,
}

impl UnInitSub {
	pub fn new(client: Client) -> Self {
		Self {
			client,
			use_best_block: false,
			block_height: Default::default(),
			poll_rate: Duration::from_secs(3),
			retry_on_error: None,
		}
	}

	pub async fn build(&self) -> Result<Sub, RpcError> {
		let block_height = match self.block_height {
			Some(x) => x,
			None => match self.use_best_block {
				true => self.client.best().block_height().await?,
				false => self.client.finalized().block_height().await?,
			},
		};

		let sub = match self.use_best_block {
			true => Sub::BestBlock(BestBlockSub {
				client: self.client.clone(),
				poll_rate: self.poll_rate,
				current_block_height: block_height,
				block_processed: Vec::new(),
				retry_on_error: self.retry_on_error,
				latest_finalized_height: None,
			}),
			false => Sub::FinalizedBlock(FinalizedBlockSub {
				client: self.client.clone(),
				poll_rate: self.poll_rate,
				next_block_height: block_height,
				retry_on_error: self.retry_on_error,
				latest_finalized_height: None,
				processed_previous_block: false,
			}),
		};

		Ok(sub)
	}
}

/// Subscription to fetch finalized block. Not meant to be used directly.
///
/// Use [Sub] instead.
#[derive(Clone)]
pub struct FinalizedBlockSub {
	client: Client,
	poll_rate: Duration,
	next_block_height: u32,
	retry_on_error: Option<bool>,
	latest_finalized_height: Option<u32>,
	processed_previous_block: bool,
}

impl FinalizedBlockSub {
	pub async fn next(&mut self) -> Result<BlockRef, RpcError> {
		let latest_finalized_height = self.fetch_latest_finalized_height().await?;

		let result = if latest_finalized_height >= self.next_block_height {
			self.run_historical().await?
		} else {
			self.run_head().await?
		};

		self.next_block_height = result.height + 1;
		self.processed_previous_block = true;
		Ok(result)
	}

	pub async fn prev(&mut self) -> Result<BlockRef, RpcError> {
		self.next_block_height = self.next_block_height.saturating_sub(1);
		if self.processed_previous_block {
			self.next_block_height = self.next_block_height.saturating_sub(1);
			self.processed_previous_block = false;
		}

		self.next().await
	}

	async fn fetch_latest_finalized_height(&mut self) -> Result<u32, RpcError> {
		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		let latest_finalized_height = self.client.finalized().block_height().await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	async fn run_historical(&mut self) -> Result<BlockRef, RpcError> {
		let retry_on_error = Some(should_retry(&self.client, self.retry_on_error));

		let height = self.next_block_height;
		let hash = self
			.client
			.rpc()
			.retry_on(retry_on_error, None)
			.block_hash(Some(height))
			.await?;
		let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

		Ok(BlockRef { hash, height })
	}

	async fn run_head(&mut self) -> Result<BlockRef, RpcError> {
		let retry_on_error = Some(should_retry(&self.client, self.retry_on_error));

		loop {
			let head = self.client.finalized().block_info().await?;

			let is_past_block = self.next_block_height > head.height;
			if is_past_block {
				sleep(self.poll_rate).await;
				continue;
			}

			if self.next_block_height == head.height {
				return Ok(head);
			}

			let height = self.next_block_height;
			let hash = self
				.client
				.rpc()
				.retry_on(retry_on_error, Some(true))
				.block_hash(Some(height))
				.await?;
			let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

			return Ok(BlockRef { hash, height });
		}
	}
}

/// Subscription to fetch best block. Not meant to be used directly.
///
/// Use [Sub] instead.
#[derive(Clone)]
pub struct BestBlockSub {
	client: Client,
	poll_rate: Duration,
	current_block_height: u32,
	block_processed: Vec<H256>,
	retry_on_error: Option<bool>,
	latest_finalized_height: Option<u32>,
}

impl BestBlockSub {
	pub async fn next(&mut self) -> Result<BlockRef, RpcError> {
		let latest_finalized_height = self.fetch_latest_finalized_height().await?;

		// Dealing with historical blocks
		if latest_finalized_height >= self.current_block_height {
			let info = self.run_historical().await?;
			self.current_block_height = info.height;
			self.block_processed.clear();
			self.block_processed.push(info.hash);
			return Ok(info);
		}

		let info = self.run_head().await?;
		if info.height == self.current_block_height {
			self.block_processed.push(info.hash);
		} else {
			self.block_processed = vec![info.hash];
			self.current_block_height = info.height;
		}

		Ok(info)
	}

	pub async fn prev(&mut self) -> Result<BlockRef, RpcError> {
		self.current_block_height = self.current_block_height.saturating_sub(1);
		self.block_processed.clear();
		self.next().await
	}

	async fn fetch_latest_finalized_height(&mut self) -> Result<u32, RpcError> {
		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		let latest_finalized_height = self.client.finalized().block_height().await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	async fn run_historical(&mut self) -> Result<BlockRef, RpcError> {
		let retry_on_error = Some(should_retry(&self.client, self.retry_on_error));

		let mut height = self.current_block_height;
		if !self.block_processed.is_empty() {
			height += 1;
		}

		let hash = self
			.client
			.rpc()
			.retry_on(retry_on_error, None)
			.block_hash(Some(height))
			.await?;
		let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

		Ok(BlockRef { hash, height })
	}

	async fn run_head(&mut self) -> Result<BlockRef, RpcError> {
		loop {
			let head = self.client.best().block_info().await?;

			let is_past_block = self.current_block_height > head.height;
			let block_already_processed = self.block_processed.contains(&head.hash);
			if is_past_block || block_already_processed {
				sleep(self.poll_rate).await;
				continue;
			}

			let is_current_block = self.current_block_height == head.height;

			let no_block_processed_yet = self.block_processed.is_empty();
			if no_block_processed_yet {
				let hash = self
					.client
					.rpc()
					.retry_on(Some(true), Some(true))
					.block_hash(Some(self.current_block_height))
					.await?;
				let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

				return Ok(BlockRef { hash, height: self.current_block_height });
			}

			let is_next_block = self.current_block_height + 1 == head.height;
			if is_current_block || is_next_block {
				return Ok(head);
			}

			let height = self.current_block_height + 1;
			let hash = self
				.client
				.rpc()
				.retry_on(Some(true), Some(true))
				.block_hash(Some(height))
				.await?;
			let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

			return Ok(BlockRef { hash, height });
		}
	}
}

/// The [Sub] subscription behaves as follows by default:
///
/// **Defaults**
/// - Tracks **finalized blocks**.  
///   → To track best (non-finalized) blocks instead, call: `sub.use_best_block(true)`
/// - Starts from the **latest** finalized (or best) block.  
///   → To start from a specific height, call: `sub.set_block_height(height)`
/// - **Retries** failed RPC calls automatically.  
///   → To disable retries, call: `sub.set_retry_on_error(false)`
/// - Polls for new block information every **3 seconds**.  
///   → To change the interval, call: `sub.set_pool_rate(Duration)`
///
/// **Fetching methods**
/// - `sub.next()` → Returns the **next block reference** `(hash, height)`.  
///   - If you’ve already fetched a block, this moves forward.  
///   - If you set a starting height, it begins from there.  
///   - Otherwise, it starts at the latest finalized (or best) block.
/// - `sub.prev()` → Returns the **previous block reference** `(hash, height)`.  
///   - If you set a starting height, it begins from `(height - 1)`.  
///   - Otherwise, it starts from `(latest finalized/best height - 1)`.
///
/// **State**
/// - The initial state is `UnInit`.  
/// - After the first call to `next()` or `prev()`, the state changes to either:  
///   - `FinalizedBlock` (default), or  
///   - `BestBlock` (if `sub.use_best_block(true)` was called).   
/// - Once initialized, calling `use_best_block(...)` has **no effect**.
///
/// # Example
/// ```rust
#[doc = include_str!("../../examples/code_doc/sub_doc.rs")]
/// ```
#[derive(Clone)]
pub enum Sub {
	UnInit(UnInitSub),
	BestBlock(BestBlockSub),
	FinalizedBlock(FinalizedBlockSub),
}

impl Sub {
	pub fn new(client: Client) -> Self {
		Self::UnInit(UnInitSub::new(client))
	}

	/// Returns the **next block reference** `(hash, height)`.  
	///	- If you’ve already called [Sub::next] or [Sub::prev] once, this moves forward.  
	///	- If you set a starting height, it begins from there.  
	///	- Otherwise, it starts at the latest finalized (or best) block.
	pub async fn next(&mut self) -> Result<BlockRef, RpcError> {
		if let Self::UnInit(u) = self {
			let concrete = u.build().await?;
			*self = concrete;
		};

		match self {
			Self::BestBlock(s) => s.next().await,
			Self::FinalizedBlock(s) => s.next().await,
			_ => unreachable!("We cannot be here."),
		}
	}

	pub async fn prev(&mut self) -> Result<BlockRef, RpcError> {
		if let Self::UnInit(u) = self {
			let concrete = u.build().await?;
			*self = concrete;
		};

		match self {
			Self::BestBlock(s) => s.prev().await,
			Self::FinalizedBlock(s) => s.prev().await,
			_ => unreachable!("We cannot be here."),
		}
	}

	pub fn should_retry_on_error(&self) -> bool {
		let value = match self {
			Self::UnInit(u) => u.retry_on_error,
			Self::BestBlock(s) => s.retry_on_error,
			Self::FinalizedBlock(s) => s.retry_on_error,
		};

		should_retry(self.client_ref(), value)
	}

	pub fn use_best_block(&mut self, value: bool) {
		if let Self::UnInit(u) = self {
			u.use_best_block = value;
		}
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		match self {
			Self::UnInit(u) => u.block_height = Some(block_height),
			Self::BestBlock(x) => {
				x.block_processed.clear();
				x.current_block_height = block_height;
			},
			Self::FinalizedBlock(x) => {
				x.next_block_height = block_height;
				x.processed_previous_block = false;
			},
		}
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		match self {
			Self::UnInit(u) => u.poll_rate = value,
			Self::BestBlock(x) => x.poll_rate = value,
			Self::FinalizedBlock(x) => x.poll_rate = value,
		}
	}

	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		match self {
			Self::UnInit(u) => u.retry_on_error = value,
			Self::BestBlock(x) => x.retry_on_error = value,
			Self::FinalizedBlock(x) => x.retry_on_error = value,
		}
	}

	fn client_ref(&self) -> &Client {
		match self {
			Sub::UnInit(x) => &x.client,
			Sub::BestBlock(x) => &x.client,
			Sub::FinalizedBlock(x) => &x.client,
		}
	}

	#[cfg(test)]
	fn as_finalized(&self) -> &FinalizedBlockSub {
		if let Self::FinalizedBlock(f) = self {
			return f;
		}
		panic!("Not Finalized Sub");
	}
}

/// The `BlockWithJustSub` subscription behaves just as [Sub]
///
/// The difference is that instead of fetching block (hash, height) it
/// fetches full blocks with justification [BlockWithJustifications].
#[derive(Clone)]
pub struct BlockWithJustSub {
	sub: Sub,
}

impl BlockWithJustSub {
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	pub async fn next(&mut self) -> Result<Option<BlockWithJustifications>, RpcError> {
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

	pub async fn prev(&mut self) -> Result<Option<BlockWithJustifications>, RpcError> {
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
pub struct TransactionSub<T: HasHeader + Decode> {
	sub: Sub,
	opts: BlockExtOptionsSimple,
	_phantom: PhantomData<T>,
}

impl<T: HasHeader + Decode> TransactionSub<T> {
	pub fn new(client: Client, opts: BlockExtOptionsSimple) -> Self {
		Self { sub: Sub::new(client), opts, _phantom: Default::default() }
	}

	pub async fn next(&mut self) -> Result<(Vec<BlockTransaction<T>>, BlockRef), crate::Error> {
		loop {
			let info = self.sub.next().await?;
			let mut block = BlockWithTx::new(self.sub.client_ref().clone(), info.hash);
			block.set_retry_on_error(Some(self.sub.should_retry_on_error()));

			let txs = match block.all::<T>(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			if txs.is_empty() {
				continue;
			}

			return Ok((txs, info));
		}
	}

	pub fn set_opts(&mut self, value: BlockExtOptionsSimple) {
		self.opts = value;
	}

	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	pub fn set_block_height(&mut self, value: u32) {
		self.sub.set_block_height(value);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct ExtrinsicSub<T: HasHeader + Decode> {
	sub: Sub,
	opts: BlockExtOptionsSimple,
	_phantom: PhantomData<T>,
}

impl<T: HasHeader + Decode> ExtrinsicSub<T> {
	pub fn new(client: Client, opts: BlockExtOptionsSimple) -> Self {
		Self { sub: Sub::new(client), opts, _phantom: Default::default() }
	}

	pub async fn next(&mut self) -> Result<(Vec<BlockExtrinsic<T>>, BlockRef), crate::Error> {
		loop {
			let info = self.sub.next().await?;
			let mut block = BlockWithExt::new(self.sub.client_ref().clone(), info.hash);
			block.set_retry_on_error(Some(self.sub.should_retry_on_error()));

			let txs = match block.all::<T>(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			if txs.is_empty() {
				continue;
			}

			return Ok((txs, info));
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

#[derive(Clone)]
pub struct RawExtrinsicSub {
	sub: Sub,
	opts: BlockExtOptionsExpanded,
}

impl RawExtrinsicSub {
	pub fn new(client: Client, opts: BlockExtOptionsExpanded) -> Self {
		Self { sub: Sub::new(client), opts }
	}

	pub async fn next(&mut self) -> Result<(Vec<BlockRawExtrinsic>, BlockRef), crate::Error> {
		loop {
			let info = self.sub.next().await?;
			let mut block = BlockWithRawExt::new(self.sub.client_ref().clone(), info.hash);
			block.set_retry_on_error(Some(self.sub.should_retry_on_error()));

			let txs = match block.all(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			if txs.is_empty() {
				continue;
			}

			return Ok((txs, info));
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

#[derive(Clone)]
pub struct GrandpaJustificationSub {
	sub: Sub,
}

impl GrandpaJustificationSub {
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let info = self.sub.next().await?;
			let retry = self.sub.should_retry_on_error();
			let just = match self.fetch_justification(info.height, retry).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			let Some(just) = just else {
				continue;
			};

			return Ok(just);
		}
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

	async fn fetch_justification(&self, height: u32, retry: bool) -> Result<Option<GrandpaJustification>, RpcError> {
		self.sub
			.client_ref()
			.rpc()
			.retry_on(Some(retry), None)
			.grandpa_block_justification(height)
			.await
	}
}

#[derive(Clone)]
pub struct GrandpaJustificationJsonSub {
	sub: Sub,
}

impl GrandpaJustificationJsonSub {
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let info = self.sub.next().await?;
			let retry = self.sub.should_retry_on_error();
			let just = match self.fetch_justification(info.height, retry).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			let Some(just) = just else {
				continue;
			};

			return Ok(just);
		}
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

	async fn fetch_justification(&self, height: u32, retry: bool) -> Result<Option<GrandpaJustification>, RpcError> {
		self.sub
			.client_ref()
			.rpc()
			.retry_on(Some(retry), None)
			.grandpa_block_justification_json(height)
			.await
	}
}

fn should_retry(client: &Client, value: Option<bool>) -> bool {
	value.unwrap_or(client.is_global_retries_enabled())
}

#[cfg(test)]
mod tests {
	use avail_rust_core::{
		avail::data_availability::tx::SubmitData, grandpa::GrandpaJustification,
		rpc::system::fetch_extrinsics::ExtrinsicInformation,
	};

	use crate::{
		block::BlockExtOptionsExpanded,
		clients::mock_client::MockClient,
		error::Error,
		prelude::*,
		subscription::{
			BlockHeaderSub, BlockSub, BlockWithJustSub, ExtrinsicSub, GrandpaJustificationJsonSub,
			GrandpaJustificationSub, RawExtrinsicSub, Sub, TransactionSub,
		},
		subxt_rpcs::RpcClient,
	};

	#[tokio::test]
	async fn grandpa_justification_sub_test() -> Result<(), Error> {
		_ = Client::init_tracing(false);
		let (rpc_client, mut commander) = MockClient::new(MAINNET_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical block
		let mut sub = GrandpaJustificationSub::new(client.clone());

		sub.set_block_height(1900031);
		let n = sub.next().await?;
		assert_eq!(n.commit.target_number, 1900032);

		sub.set_block_height(1900122);
		let n = sub.next().await?;
		assert_eq!(n.commit.target_number, 1900122);

		// Testing recovery
		sub.set_block_height(1);
		assert_eq!(sub.sub.as_finalized().next_block_height, 1);

		// 1 is Ok(Some)
		// 2 is Ok(None)
		// 3 is Ok(Some)
		// 4 is Err
		// 4 is Ok(Some)
		commander.justification_ok(Some(GrandpaJustification::default())); // 1
		commander.justification_ok(None); // 2
		commander.justification_ok(Some(GrandpaJustification::default())); // 3
		commander.justification_err(None); // 4
		commander.justification_ok(Some(GrandpaJustification::default())); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(Some(false));
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}

	#[tokio::test]
	async fn grandpa_justification_json_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(MAINNET_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical block
		let mut sub = GrandpaJustificationJsonSub::new(client.clone());

		sub.set_block_height(1900031);
		let n = sub.next().await?;
		assert_eq!(n.commit.target_number, 1900032);

		sub.set_block_height(1900122);
		let n = sub.next().await?;
		assert_eq!(n.commit.target_number, 1900122);

		// Testing recovery
		sub.set_block_height(1);
		assert_eq!(sub.sub.as_finalized().next_block_height, 1);

		// 1 is Ok(Some)
		// 2 is Ok(None)
		// 3 is Ok(Some)
		// 4 is Err
		// 4 is Ok(Some)
		commander.justification_json_ok(Some(GrandpaJustification::default())); // 1
		commander.justification_json_ok(None); // 2
		commander.justification_json_ok(Some(GrandpaJustification::default())); // 3
		commander.justification_json_err(None); // 4
		commander.justification_json_ok(Some(GrandpaJustification::default())); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(Some(false));
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}

	#[tokio::test]
	async fn extrinsic_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical blocks
		let mut sub = ExtrinsicSub::<SubmitData>::new(client.clone(), Default::default());

		sub.set_block_height(2326671);
		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326672);
		assert_eq!(list.len(), 1);

		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326674);
		assert_eq!(list.len(), 1);

		// Testing recovery
		sub.set_block_height(1);
		assert_eq!(sub.sub.as_finalized().next_block_height, 1);

		// 1 is Ok(Some)
		// 2 is Ok(None)
		// 3 is Ok(Some)
		// 4 is Err
		// 4 is Ok(Some)
		let mut data = ExtrinsicInformation::default();
		let tx = client.tx().data_availability().submit_data("1234");
		data.encoded = Some(const_hex::encode(tx.sign(&alice(), Options::new(2)).await?.encode()));

		commander.extrinsics_ok(vec![data.clone()]); // 1
		commander.extrinsics_ok(vec![]); // 2
		commander.extrinsics_ok(vec![data.clone()]); // 3
		commander.extrinsics_err(None); // 4
		commander.extrinsics_ok(vec![data.clone()]); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(Some(false));
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}

	#[tokio::test]
	async fn transaction_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical blocks
		let mut sub = TransactionSub::<SubmitData>::new(client.clone(), Default::default());

		sub.set_block_height(2326671);
		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326672);
		assert_eq!(list.len(), 1);

		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326674);
		assert_eq!(list.len(), 1);

		// Testing recovery
		sub.set_block_height(1);
		assert_eq!(sub.sub.as_finalized().next_block_height, 1);

		// 1 is Ok(Some)
		// 2 is Ok(None)
		// 3 is Ok(Some)
		// 4 is Err
		// 4 is Ok(Some)
		let mut data = ExtrinsicInformation::default();
		let tx = client.tx().data_availability().submit_data("1234");
		data.encoded = Some(const_hex::encode(tx.sign(&alice(), Options::new(2)).await?.encode()));

		commander.extrinsics_ok(vec![data.clone()]); // 1
		commander.extrinsics_ok(vec![]); // 2
		commander.extrinsics_ok(vec![data.clone()]); // 3
		commander.extrinsics_err(None); // 4
		commander.extrinsics_ok(vec![data.clone()]); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(Some(false));
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}

	#[tokio::test]
	async fn raw_extrinsic_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical blocks
		let opts = BlockExtOptionsExpanded { filter: Some((29u8, 1u8).into()), ..Default::default() };
		let mut sub = RawExtrinsicSub::new(client.clone(), opts);

		sub.set_block_height(2326671);
		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326672);
		assert_eq!(list.len(), 1);

		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326674);
		assert_eq!(list.len(), 1);

		// Testing recovery
		sub.set_block_height(1);
		assert_eq!(sub.sub.as_finalized().next_block_height, 1);

		// 1 is Ok(Some)
		// 2 is Ok(None)
		// 3 is Ok(Some)
		// 4 is Err
		// 4 is Ok(Some)
		let mut data = ExtrinsicInformation::default();
		let tx = client.tx().data_availability().submit_data("1234");
		data.encoded = Some(const_hex::encode(tx.sign(&alice(), Options::new(2)).await?.encode()));

		commander.extrinsics_ok(vec![data.clone()]); // 1
		commander.extrinsics_ok(vec![]); // 2
		commander.extrinsics_ok(vec![data.clone()]); // 3
		commander.extrinsics_err(None); // 4
		commander.extrinsics_ok(vec![data.clone()]); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(Some(false));
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}

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
		let mut sub = BlockWithJustSub::new(client.clone());
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
		let mut sub = BlockWithJustSub::new(client.clone());
		sub.use_best_block(true);

		let block = sub.next().await?.unwrap();
		assert_eq!(block.block.header.number, expected);

		// Finalized Block
		let expected = client.finalized().block_height().await?;
		let mut sub = BlockWithJustSub::new(client);
		sub.use_best_block(false);

		let block = sub.next().await?.unwrap();
		assert_eq!(block.block.header.number, expected);

		Ok(())
	}

	// This test will be by flaky and that is OK.
	#[tokio::test]
	pub async fn best_sub_test() -> Result<(), Error> {
		let (rpc_client, _commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical block
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(true);
		sub.set_block_height(1900000);

		let block = sub.prev().await?;
		assert_eq!(block.height, 1899999);

		let block = sub.next().await?;
		assert_eq!(block.height, 1900000);

		let block = sub.next().await?;
		assert_eq!(block.height, 1900001);

		let block = sub.prev().await?;
		assert_eq!(block.height, 1900000);

		let block = sub.prev().await?;
		assert_eq!(block.height, 1899999);

		// Historical block #2
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(true);
		sub.set_block_height(1900000);

		let block = sub.next().await?;
		assert_eq!(block.height, 1900000);

		// Latest Block #1
		let best_height = client.best().block_height().await?;
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(true);
		let block = sub.prev().await?;
		assert_eq!(block.height, best_height - 1);

		// Latest Block #2
		let best_height = client.best().block_height().await?;
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(true);
		let block = sub.next().await?;
		assert_eq!(block.height, best_height);

		Ok(())
	}

	// This test will be by flaky and that is OK.
	#[tokio::test]
	pub async fn finalized_sub_test() -> Result<(), Error> {
		let (rpc_client, _commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical block
		let mut sub = Sub::new(client.clone());
		sub.set_block_height(1900000);

		let block = sub.prev().await?;
		assert_eq!(block.height, 1899999);

		let block = sub.next().await?;
		assert_eq!(block.height, 1900000);

		let block = sub.next().await?;
		assert_eq!(block.height, 1900001);

		let block = sub.prev().await?;
		assert_eq!(block.height, 1900000);

		let block = sub.prev().await?;
		assert_eq!(block.height, 1899999);

		// Historical block #2
		let mut sub = Sub::new(client.clone());
		sub.set_block_height(1900000);

		let block = sub.next().await?;
		assert_eq!(block.height, 1900000);

		// Latest Block #1
		let finalized_height = client.finalized().block_height().await?;
		let mut sub = Sub::new(client.clone());
		let block = sub.prev().await?;
		assert_eq!(block.height, finalized_height - 1);

		// Latest Block #2
		let finalized_height = client.finalized().block_height().await?;
		let mut sub = Sub::new(client.clone());
		let block = sub.next().await?;
		assert_eq!(block.height, finalized_height);

		Ok(())
	}
}
