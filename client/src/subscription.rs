use crate::{AvailHeader, Client, platform::sleep};
use avail_rust_core::{BlockRef, H256, RpcError, grandpa::GrandpaJustification, rpc::BlockWithJustifications};
use std::time::Duration;

#[derive(Debug, Default, Clone, Copy)]
pub enum SubscriptionKind {
	BestBlock,
	#[default]
	FinalizedBlock,
}

#[derive(Clone)]
pub struct SubscriptionBuilder {
	kind: SubscriptionKind,
	block_height: Option<u32>,
	poll_rate: Duration,
	retry_on_error: bool,
}

impl SubscriptionBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn follow(mut self, best_block: bool) -> Self {
		if best_block {
			self.kind = SubscriptionKind::BestBlock;
		} else {
			self.kind = SubscriptionKind::FinalizedBlock;
		}
		self
	}

	pub fn follow_best_blocks(mut self) -> Self {
		self.kind = SubscriptionKind::BestBlock;
		self
	}

	pub fn follow_finalized_blocks(mut self) -> Self {
		self.kind = SubscriptionKind::FinalizedBlock;
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

	pub fn kind(mut self, value: SubscriptionKind) -> Self {
		self.kind = value;
		self
	}

	pub fn retry_on_error(mut self, value: bool) -> Self {
		self.retry_on_error = value;
		self
	}

	pub async fn build(&self, client: &Client) -> Result<Subscription, RpcError> {
		let block_height = match self.block_height {
			Some(x) => x,
			None => match self.kind {
				SubscriptionKind::BestBlock => client.best().block_height().await?,
				SubscriptionKind::FinalizedBlock => client.finalized().block_height().await?,
			},
		};

		let sub = match self.kind {
			SubscriptionKind::BestBlock => Subscription::BestBlock(BestBlockSubscriber {
				poll_rate: self.poll_rate,
				current_block_height: block_height,
				block_processed: Vec::new(),
				retry_on_error: self.retry_on_error,
				latest_finalized_height: None,
			}),
			SubscriptionKind::FinalizedBlock => Subscription::FinalizedBlock(FinalizedBlockSubscriber {
				poll_rate: self.poll_rate,
				next_block_height: block_height,
				retry_on_error: self.retry_on_error,
				latest_finalized_height: None,
			}),
		};
		Ok(sub)
	}
}

impl Default for SubscriptionBuilder {
	fn default() -> Self {
		Self {
			kind: Default::default(),
			block_height: Default::default(),
			poll_rate: Duration::from_secs(3),
			retry_on_error: true,
		}
	}
}

#[derive(Clone)]
pub struct FinalizedBlockSubscriber {
	poll_rate: Duration,
	next_block_height: u32,
	retry_on_error: bool,
	latest_finalized_height: Option<u32>,
}

impl FinalizedBlockSubscriber {
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
pub struct BestBlockSubscriber {
	poll_rate: Duration,
	current_block_height: u32,
	block_processed: Vec<H256>,
	retry_on_error: bool,
	latest_finalized_height: Option<u32>,
}

impl BestBlockSubscriber {
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
pub enum Subscription {
	BestBlock(BestBlockSubscriber),
	FinalizedBlock(FinalizedBlockSubscriber),
}

impl Subscription {
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
}

#[derive(Clone)]
pub struct HeaderSubscription {
	client: Client,
	sub: Subscription,
	retry_on_error: bool,
}

impl HeaderSubscription {
	pub fn new(client: Client, sub: Subscription) -> Self {
		let retry_on_error = sub.retry_on_error();
		Self { client, sub, retry_on_error }
	}

	pub async fn next(&mut self) -> Result<Option<AvailHeader>, RpcError> {
		let info = self.sub.next(&self.client).await?;
		self.client
			.rpc()
			.retry_on(Some(self.retry_on_error), Some(true))
			.block_header(Some(info.hash))
			.await
	}

	pub fn current_block_height(&self) -> u32 {
		self.sub.current_block_height()
	}
}

#[derive(Clone)]
pub struct BlockSubscription {
	client: Client,
	sub: Subscription,
	retry_on_error: bool,
}

impl BlockSubscription {
	pub fn new(client: Client, sub: Subscription) -> Self {
		let retry_on_error = sub.retry_on_error();
		Self { client, sub, retry_on_error }
	}

	pub async fn next(&mut self) -> Result<Option<BlockWithJustifications>, RpcError> {
		let info = self.sub.next(&self.client).await?;
		self.client
			.rpc()
			.retry_on(Some(self.retry_on_error), Some(true))
			.block(Some(info.hash))
			.await
	}

	pub fn current_block_height(&self) -> u32 {
		self.sub.current_block_height()
	}
}

#[derive(Clone)]
pub struct GrandpaJustificationSubscription {
	client: Client,
	next_block_height: u32,
	poll_rate: Duration,
	latest_finalized_height: Option<u32>,
	stopwatch: std::time::Instant,
}

impl GrandpaJustificationSubscription {
	pub fn new(client: Client, poll_rate: Duration, next_block_height: u32) -> Self {
		Self {
			client,
			next_block_height,
			poll_rate,
			latest_finalized_height: None,
			stopwatch: std::time::Instant::now(),
		}
	}

	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let latest_finalized_height = self.fetch_latest_finalized_height().await?;

			// Dealing with historical blocks
			let block_height = if latest_finalized_height > self.next_block_height {
				self.run_historical()
			} else {
				self.run_head().await?
			};

			let justification = self.client.rpc().grandpa_block_justification(block_height).await?;
			self.next_block_height += 1;

			let Some(justification) = justification else {
				continue;
			};

			return Ok(justification);
		}
	}

	async fn fetch_latest_finalized_height(&mut self) -> Result<u32, RpcError> {
		if self.stopwatch.elapsed().as_secs() > 300 {
			self.latest_finalized_height = None
		}

		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		self.stopwatch = std::time::Instant::now();
		let latest_finalized_height = self.client.finalized().block_height().await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	fn run_historical(&mut self) -> u32 {
		self.next_block_height
	}

	async fn run_head(&mut self) -> Result<u32, RpcError> {
		loop {
			let head = self.client.finalized().block_info().await?;
			if self.next_block_height > head.height {
				sleep(self.poll_rate).await;
				continue;
			}
			return Ok(self.next_block_height);
		}
	}
}

#[derive(Clone)]
pub struct GrandpaJustificationJsonSubscription {
	client: Client,
	next_block_height: u32,
	poll_rate: Duration,
	latest_finalized_height: Option<u32>,
	stopwatch: std::time::Instant,
}

impl GrandpaJustificationJsonSubscription {
	pub fn new(client: Client, poll_rate: Duration, next_block_height: u32) -> Self {
		Self {
			client,
			next_block_height,
			poll_rate,
			latest_finalized_height: None,
			stopwatch: std::time::Instant::now(),
		}
	}

	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let latest_finalized_height = self.fetch_latest_finalized_height().await?;

			// Dealing with historical blocks
			let block_height = if latest_finalized_height > self.next_block_height {
				self.run_historical()
			} else {
				self.run_head().await?
			};

			let justification = self.client.rpc().grandpa_block_justification_json(block_height).await?;
			self.next_block_height += 1;

			let Some(justification) = justification else {
				continue;
			};

			return Ok(justification);
		}
	}

	async fn fetch_latest_finalized_height(&mut self) -> Result<u32, RpcError> {
		if self.stopwatch.elapsed().as_secs() > 300 {
			self.latest_finalized_height = None
		}

		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		self.stopwatch = std::time::Instant::now();
		let latest_finalized_height = self.client.finalized().block_height().await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	fn run_historical(&mut self) -> u32 {
		self.next_block_height
	}

	async fn run_head(&mut self) -> Result<u32, RpcError> {
		loop {
			let head = self.client.finalized().block_info().await?;
			if self.next_block_height > head.height {
				sleep(self.poll_rate).await;
				continue;
			}
			return Ok(self.next_block_height);
		}
	}
}
