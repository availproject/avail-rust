use super::fetcher::Fetcher;
use crate::{BlockInfo, Client, Error, H256, RetryPolicy, RpcError, platform::sleep};
use futures::stream::{self, Stream};
use std::time::Duration;

/// Selects whether subscriptions follow best blocks or finalized blocks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BlockQueryMode {
	/// Follow finalized blocks only.
	#[default]
	Finalized,
	/// Follow best (head) blocks, including non-finalized ones.
	Best,
}

#[derive(Debug, Clone)]
pub(crate) struct SubConfig {
	pub mode: BlockQueryMode,
	pub start_height: Option<u32>,
	pub poll_interval: Duration,
	pub retry_policy: RetryPolicy,
}

impl Default for SubConfig {
	fn default() -> Self {
		Self {
			mode: BlockQueryMode::Finalized,
			start_height: None,
			poll_interval: Duration::from_secs(3),
			retry_policy: RetryPolicy::Inherit,
		}
	}
}

pub(crate) enum Sub {
	BestBlock(BestBlockSub),
	FinalizedBlock(FinalizedBlockSub),
}

impl Sub {
	pub(crate) async fn init(client: Client, config: SubConfig) -> Result<Self, RpcError> {
		let height = match config.start_height {
			Some(h) => h,
			None => match config.mode {
				BlockQueryMode::Best => client.best().block_height().await?,
				BlockQueryMode::Finalized => client.finalized().block_height().await?,
			},
		};

		let sub = match config.mode {
			BlockQueryMode::Best => Sub::BestBlock(BestBlockSub {
				client,
				poll_rate: config.poll_interval,
				current_block_height: height,
				block_processed: Vec::new(),
				retry_on_error: config.retry_policy,
			}),
			BlockQueryMode::Finalized => Sub::FinalizedBlock(FinalizedBlockSub {
				client,
				poll_rate: config.poll_interval,
				next_block_height: height,
				retry_on_error: config.retry_policy,
				processed_previous_block: false,
			}),
		};

		Ok(sub)
	}

	pub(crate) async fn next(&mut self) -> Result<BlockInfo, RpcError> {
		match self {
			Self::BestBlock(s) => s.next().await,
			Self::FinalizedBlock(s) => s.next().await,
		}
	}

	pub(crate) async fn prev(&mut self) -> Result<BlockInfo, RpcError> {
		match self {
			Self::BestBlock(s) => s.prev().await,
			Self::FinalizedBlock(s) => s.prev().await,
		}
	}

	pub(crate) fn set_block_height(&mut self, value: u32) {
		match self {
			Self::BestBlock(x) => {
				x.current_block_height = value;
				x.block_processed.clear();
			},
			Self::FinalizedBlock(x) => {
				x.next_block_height = value;
				x.processed_previous_block = false;
			},
		}
	}

	pub(crate) fn client_ref(&self) -> &Client {
		match self {
			Sub::BestBlock(x) => &x.client,
			Sub::FinalizedBlock(x) => &x.client,
		}
	}

	pub(crate) fn resolved_retry_policy(&self) -> RetryPolicy {
		let policy = match self {
			Self::BestBlock(s) => s.retry_on_error,
			Self::FinalizedBlock(s) => s.retry_on_error,
		};
		if should_retry(self.client_ref(), policy) {
			RetryPolicy::Enabled
		} else {
			RetryPolicy::Disabled
		}
	}
}

fn should_retry(client: &Client, policy: RetryPolicy) -> bool {
	policy.resolve(client.retry_policy() != RetryPolicy::Disabled)
}

// ---------------------------------------------------------------------------
// Finalized block cursor
// ---------------------------------------------------------------------------

pub(crate) struct FinalizedBlockSub {
	client: Client,
	poll_rate: Duration,
	pub(crate) next_block_height: u32,
	retry_on_error: RetryPolicy,
	processed_previous_block: bool,
}

impl FinalizedBlockSub {
	fn retry_policy(&self) -> RetryPolicy {
		if should_retry(&self.client, self.retry_on_error) {
			RetryPolicy::Enabled
		} else {
			RetryPolicy::Disabled
		}
	}

	fn chain(&self, none: RetryPolicy) -> crate::chain::Chain {
		self.client.chain().retry_policy(self.retry_on_error, none)
	}

	pub async fn next(&mut self) -> Result<BlockInfo, RpcError> {
		let latest_finalized_height = self.fetch_latest_finalized_height().await?;

		let (hash, height) = if latest_finalized_height > self.next_block_height {
			self.run_historical().await?
		} else {
			self.run_head().await?
		};

		self.next_block_height = height + 1;
		self.processed_previous_block = true;
		Ok(BlockInfo { hash, height })
	}

	pub async fn prev(&mut self) -> Result<BlockInfo, RpcError> {
		self.next_block_height = self.next_block_height.saturating_sub(1);
		if self.processed_previous_block {
			self.next_block_height = self.next_block_height.saturating_sub(1);
		}
		self.processed_previous_block = false;

		self.next().await
	}

	async fn fetch_latest_finalized_height(&self) -> Result<u32, RpcError> {
		self.client
			.finalized()
			.retry_policy(self.retry_policy())
			.block_height()
			.await
	}

	async fn run_historical(&mut self) -> Result<(H256, u32), RpcError> {
		let height = self.next_block_height;
		let hash = self.chain(RetryPolicy::Inherit).block_hash(Some(height)).await?;
		let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

		Ok((hash, height))
	}

	async fn run_head(&mut self) -> Result<(H256, u32), RpcError> {
		loop {
			let head = self.chain(RetryPolicy::Inherit).info().await?;

			let is_past_block = self.next_block_height > head.finalized_height;
			if is_past_block {
				sleep(self.poll_rate).await;
				continue;
			}

			if self.next_block_height == head.finalized_height {
				return Ok((head.finalized_hash, head.finalized_height));
			}

			let height = self.next_block_height;
			let hash = self.chain(RetryPolicy::Enabled).block_hash(Some(height)).await?;
			let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

			return Ok((hash, height));
		}
	}
}

// ---------------------------------------------------------------------------
// Best block cursor
// ---------------------------------------------------------------------------

pub(crate) struct BestBlockSub {
	client: Client,
	poll_rate: Duration,
	pub(crate) current_block_height: u32,
	block_processed: Vec<H256>,
	retry_on_error: RetryPolicy,
}

impl BestBlockSub {
	fn chain(&self, none: RetryPolicy) -> crate::chain::Chain {
		self.client.chain().retry_policy(self.retry_on_error, none)
	}

	pub async fn next(&mut self) -> Result<BlockInfo, RpcError> {
		let latest_finalized_height = self.fetch_latest_finalized_height().await?;

		if latest_finalized_height > self.current_block_height {
			let info = self.run_historical().await?;
			self.block_processed.clear();
			self.block_processed.push(info.hash);
			self.current_block_height = info.height;
			return Ok(info);
		}

		let (hash, height) = self.run_head().await?;
		if height == self.current_block_height {
			self.block_processed.push(hash);
		} else {
			self.block_processed.clear();
			self.block_processed.push(hash);
			self.current_block_height = height;
		}

		Ok(BlockInfo { hash, height })
	}

	pub async fn prev(&mut self) -> Result<BlockInfo, RpcError> {
		self.current_block_height = self.current_block_height.saturating_sub(1);
		self.block_processed.clear();
		self.next().await
	}

	async fn fetch_latest_finalized_height(&self) -> Result<u32, RpcError> {
		self.client
			.finalized()
			.retry_policy(self.retry_on_error)
			.block_height()
			.await
	}

	async fn run_historical(&mut self) -> Result<BlockInfo, RpcError> {
		let mut height = self.current_block_height;
		if !self.block_processed.is_empty() {
			height += 1;
		}

		let hash = self.chain(RetryPolicy::Inherit).block_hash(Some(height)).await?;
		let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

		Ok(BlockInfo { hash, height })
	}

	async fn run_head(&mut self) -> Result<(H256, u32), RpcError> {
		loop {
			let head = self.chain(RetryPolicy::Inherit).info().await?;

			let is_past_block = self.current_block_height > head.best_height;
			let block_already_processed = self.block_processed.contains(&head.best_hash);
			if is_past_block || block_already_processed {
				sleep(self.poll_rate).await;
				continue;
			}

			let no_block_processed_yet = self.block_processed.is_empty();
			if no_block_processed_yet {
				let hash = self
					.chain(RetryPolicy::Enabled)
					.block_hash(Some(self.current_block_height))
					.await?;
				let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

				return Ok((hash, self.current_block_height));
			}

			let is_current_block = self.current_block_height == head.best_height;
			let is_next_block = self.current_block_height + 1 == head.best_height;
			if is_current_block || is_next_block {
				return Ok((head.best_hash, head.best_height));
			}

			let height = self.current_block_height + 1;
			let hash = self.chain(RetryPolicy::Enabled).block_hash(Some(height)).await?;
			let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

			return Ok((hash, height));
		}
	}
}

#[derive(Debug, Clone)]
pub struct SubscriptionItem<T> {
	pub value: T,
	pub block_height: u32,
	pub block_hash: H256,
}

pub struct Subscription<F: Fetcher> {
	pub(super) sub: Sub,
	pub(super) fetcher: F,
	pub(super) skip_empty: bool,
}

impl<F: Fetcher> Subscription<F> {
	pub async fn next(&mut self) -> Result<SubscriptionItem<F::Output>, Error> {
		loop {
			let info = self.sub.next().await?;
			match self.fetch_at(info).await {
				Ok(Some(item)) => return Ok(item),
				Ok(None) => continue,
				Err(e) => return Err(e),
			}
		}
	}

	pub async fn prev(&mut self) -> Result<SubscriptionItem<F::Output>, Error> {
		loop {
			let info = self.sub.prev().await?;
			match self.fetch_at(info).await {
				Ok(Some(item)) => return Ok(item),
				Ok(None) => continue,
				Err(e) => return Err(e),
			}
		}
	}

	pub fn into_stream(self) -> impl Stream<Item = Result<SubscriptionItem<F::Output>, Error>> {
		stream::try_unfold(self, |mut this| async move {
			let item = this.next().await?;
			Ok(Some((item, this)))
		})
	}

	async fn fetch_at(&mut self, info: BlockInfo) -> Result<Option<SubscriptionItem<F::Output>>, Error> {
		let client = self.sub.client_ref().clone();
		let retry = self.sub.resolved_retry_policy();

		match self.fetcher.fetch(&client, info, retry).await {
			Ok(value) => {
				if self.skip_empty && self.fetcher.is_empty(&value) {
					return Ok(None);
				}
				Ok(Some(SubscriptionItem { value, block_height: info.height, block_hash: info.hash }))
			},
			Err(e) => {
				self.sub.set_block_height(info.height);
				Err(e)
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{error::Error, prelude::*};

	#[tokio::test]
	async fn sub_init_finalized() -> Result<(), Error> {
		let client = Client::connect(TURING_ENDPOINT).await?;
		let config = SubConfig::default();
		let mut sub = Sub::init(client.clone(), config).await?;

		let block_height = client.finalized().block_height().await?;
		let value = sub.next().await?;
		assert_eq!(value.height, block_height);

		Ok(())
	}

	#[tokio::test]
	async fn sub_init_best() -> Result<(), Error> {
		let client = Client::connect(TURING_ENDPOINT).await?;
		let config = SubConfig { mode: BlockQueryMode::Best, ..Default::default() };
		let mut sub = Sub::init(client.clone(), config).await?;

		let block_height = client.best().block_height().await?;
		let value = sub.next().await?;
		assert_eq!(value.height, block_height);

		Ok(())
	}

	#[tokio::test]
	async fn sub_historical_next() -> Result<(), Error> {
		let client = Client::connect(TURING_ENDPOINT).await?;
		let block_height = 1900000u32;
		let config = SubConfig { start_height: Some(block_height), ..Default::default() };
		let mut sub = Sub::init(client, config).await?;
		for i in 0..3 {
			let value = sub.next().await?;
			assert_eq!(value.height, block_height + i);
		}

		Ok(())
	}

	#[tokio::test]
	async fn sub_historical_prev() -> Result<(), Error> {
		let client = Client::connect(TURING_ENDPOINT).await?;
		let block_height = 1900000u32;
		let config = SubConfig { start_height: Some(block_height), ..Default::default() };
		let mut sub = Sub::init(client, config).await?;
		for i in 0..3 {
			let value = sub.prev().await?;
			assert_eq!(value.height, block_height - i - 1);
		}

		Ok(())
	}
}
