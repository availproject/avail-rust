use crate::{AvailHeader, Client, platform::sleep};
use avail_rust_core::{Error as CoreError, H256, grandpa::GrandpaJustification, rpc::BlockWithJustifications};
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

	pub async fn build(&self, client: &Client) -> Result<Subscription, CoreError> {
		let block_height = match self.block_height {
			Some(x) => x,
			None => match self.kind {
				SubscriptionKind::BestBlock => client.best_block_height_ext(self.retry_on_error).await?,
				SubscriptionKind::FinalizedBlock => {
					client.finalized_block_height_ext(self.retry_on_error, true).await?
				},
			},
		};

		let sub = match self.kind {
			SubscriptionKind::BestBlock => Subscription::BestBlock(BestBlockSubscriber {
				poll_rate: self.poll_rate,
				current_block_height: block_height,
				block_processed: Vec::new(),
				retry_on_error: self.retry_on_error,
				latest_finalized_height: None,
				stopwatch: std::time::Instant::now(),
			}),
			SubscriptionKind::FinalizedBlock => Subscription::FinalizedBlock(FinalizedBlockSubscriber {
				poll_rate: self.poll_rate,
				next_block_height: block_height,
				retry_on_error: self.retry_on_error,
				latest_finalized_height: None,
				stopwatch: std::time::Instant::now(),
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
	stopwatch: std::time::Instant,
}

impl FinalizedBlockSubscriber {
	pub async fn run(&mut self, client: &Client) -> Result<Option<(u32, H256)>, CoreError> {
		let latest_finalized_height = self.fetch_latest_finalized_height(client).await?;

		// Dealing with historical blocks
		if latest_finalized_height > self.next_block_height {
			return self.run_historical(client).await;
		}

		// Dealing with most recent blocks
		self.run_head(client).await.map(Some)
	}

	pub fn current_block_height(&self) -> u32 {
		self.next_block_height.saturating_sub(1)
	}

	async fn fetch_latest_finalized_height(&mut self, client: &Client) -> Result<u32, CoreError> {
		if self.stopwatch.elapsed().as_secs() > 300 {
			self.latest_finalized_height = None
		}

		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		self.stopwatch = std::time::Instant::now();
		let latest_finalized_height = client.finalized_block_height_ext(self.retry_on_error, true).await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	async fn run_historical(&mut self, client: &Client) -> Result<Option<(u32, H256)>, CoreError> {
		let block_height = self.next_block_height;
		let block_hash = client.block_hash_ext(block_height, self.retry_on_error, false).await?;
		self.next_block_height = block_height + 1;

		Ok(block_hash.map(|x| (block_height, x)))
	}

	async fn run_head(&mut self, client: &Client) -> Result<(u32, H256), CoreError> {
		loop {
			let head = client.finalized_block_loc_ext(self.retry_on_error, true).await?;
			if self.next_block_height > head.height {
				sleep(self.poll_rate).await;
				continue;
			}
			self.next_block_height += 1;
			return Ok((head.height, head.hash));
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
	stopwatch: std::time::Instant,
}

impl BestBlockSubscriber {
	pub async fn run(&mut self, client: &Client) -> Result<Option<(u32, H256)>, CoreError> {
		let latest_finalized_height = self.fetch_latest_finalized_height(client).await?;

		// Dealing with historical blocks
		if latest_finalized_height > self.current_block_height {
			return self.run_historical(client).await;
		}

		// Dealing with most recent blocks
		self.run_head(client).await.map(Some)
	}

	pub fn current_block_height(&self) -> u32 {
		self.current_block_height
	}

	async fn fetch_latest_finalized_height(&mut self, client: &Client) -> Result<u32, CoreError> {
		if self.stopwatch.elapsed().as_secs() > 300 {
			self.latest_finalized_height = None
		}

		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		self.stopwatch = std::time::Instant::now();
		let latest_finalized_height = client.finalized_block_height_ext(self.retry_on_error, true).await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	async fn run_historical(&mut self, client: &Client) -> Result<Option<(u32, H256)>, CoreError> {
		let block_height = self.current_block_height;
		let block_hash = client.block_hash_ext(block_height, self.retry_on_error, false).await?;
		self.current_block_height = block_height + 1;
		self.block_processed.clear();

		Ok(block_hash.map(|x| (block_height, x)))
	}

	async fn run_head(&mut self, client: &Client) -> Result<(u32, H256), CoreError> {
		loop {
			let head_hash = client.best_block_hash_ext(self.retry_on_error, true).await?;
			if self.block_processed.contains(&head_hash) {
				sleep(self.poll_rate).await;
				continue;
			}

			let head_height = client.block_height_ext(head_hash, self.retry_on_error, true).await?;
			let Some(head_height) = head_height else {
				return Err(CoreError::from("Failed to fetch block height"));
			};

			let is_current_block = head_height == self.current_block_height;
			if is_current_block {
				self.block_processed.push(head_hash);
				return Ok((head_height, head_hash));
			}

			let is_ahead_of_current_block = head_height > self.current_block_height;
			let is_next_block = head_height == (self.current_block_height + 1);
			let no_block_processed_yet = self.block_processed.is_empty();

			if is_ahead_of_current_block {
				if no_block_processed_yet {
					let block_hash = client
						.block_hash_ext(self.current_block_height, self.retry_on_error, true)
						.await?;
					let Some(block_hash) = block_hash else {
						return Err(CoreError::from("Failed to fetch block hash"));
					};

					self.block_processed.push(head_hash);
					return Ok((self.current_block_height, block_hash));
				}

				if is_next_block {
					self.current_block_height = head_height;
					self.block_processed.clear();
					self.block_processed.push(head_hash);
					return Ok((head_height, head_hash));
				}

				let next_block_height = self.current_block_height + 1;
				let block_hash = client
					.block_hash_ext(next_block_height, self.retry_on_error, true)
					.await?;
				let Some(block_hash) = block_hash else {
					return Err(CoreError::from("Failed to fetch block hash"));
				};

				self.block_processed.clear();
				self.block_processed.push(block_hash);
				self.current_block_height = next_block_height;
				return Ok((next_block_height, block_hash));
			}

			// If we are here it means we are targeting a block in the future
			sleep(self.poll_rate).await;
			continue;
		}
	}
}

#[derive(Clone)]
pub enum Subscription {
	BestBlock(BestBlockSubscriber),
	FinalizedBlock(FinalizedBlockSubscriber),
}

impl Subscription {
	pub async fn run(&mut self, client: &Client) -> Result<Option<(u32, H256)>, CoreError> {
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

	pub async fn next(&mut self) -> Result<Option<AvailHeader>, CoreError> {
		let block_info = self.sub.run(&self.client).await?;

		let Some((_, block_hash)) = block_info else {
			return Ok(None);
		};

		self.client
			.block_header_ext(block_hash, self.retry_on_error, true)
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

	pub async fn next(&mut self) -> Result<Option<BlockWithJustifications>, CoreError> {
		let block_info = self.sub.run(&self.client).await?;

		let Some((_, block_hash)) = block_info else {
			return Ok(None);
		};

		self.client.block_ext(block_hash, self.retry_on_error, true).await
	}

	pub fn current_block_height(&self) -> u32 {
		self.sub.current_block_height()
	}
}

// This one is a bit different.
#[derive(Clone)]
pub struct GrandpaJustificationsSubscription {
	client: Client,
	next_block_height: u32,
	poll_rate: Duration,
	latest_finalized_height: Option<u32>,
	stopwatch: std::time::Instant,
}

impl GrandpaJustificationsSubscription {
	pub fn new(client: Client, poll_rate: Duration, next_block_height: u32) -> Self {
		Self {
			client,
			next_block_height,
			poll_rate,
			latest_finalized_height: None,
			stopwatch: std::time::Instant::now(),
		}
	}

	pub async fn next(&mut self) -> Result<(GrandpaJustification, u32), CoreError> {
		loop {
			let latest_finalized_height = self.fetch_latest_finalized_height().await?;

			// Dealing with historical blocks
			let block_height = if latest_finalized_height > self.next_block_height {
				self.run_historical()
			} else {
				self.run_head().await?
			};

			let justification = self.client.rpc_api().grandpa_block_justification(block_height).await?;
			self.next_block_height += 1;

			let Some(justification) = justification else {
				continue;
			};

			return Ok((justification, block_height));
		}
	}

	async fn fetch_latest_finalized_height(&mut self) -> Result<u32, CoreError> {
		if self.stopwatch.elapsed().as_secs() > 300 {
			self.latest_finalized_height = None
		}

		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		self.stopwatch = std::time::Instant::now();
		let latest_finalized_height = self.client.finalized_block_height_ext(true, true).await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	fn run_historical(&mut self) -> u32 {
		self.next_block_height
	}

	async fn run_head(&mut self) -> Result<u32, CoreError> {
		loop {
			let head = self.client.finalized_block_loc_ext(true, true).await?;
			if self.next_block_height > head.height {
				sleep(self.poll_rate).await;
				continue;
			}
			return Ok(self.next_block_height);
		}
	}
}
