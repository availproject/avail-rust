use super::should_retry;
use crate::{BlockRef, Client, H256, RpcError, platform::sleep};
use std::time::Duration;

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
#[doc = include_str!("../../../examples/code_doc/sub_doc.rs")]
/// ```
#[derive(Clone)]
pub enum Sub {
	UnInit(UnInitSub),
	BestBlock(BestBlockSub),
	FinalizedBlock(FinalizedBlockSub),
}

impl Sub {
	/// Creates a new lazy subscription using the provided `client`.
	pub fn new(client: Client) -> Self {
		Self::UnInit(UnInitSub::new(client))
	}

	/// Returns the **next block reference** `(hash, height)`.  
	///	- If you’ve already called [Sub::next] or [Sub::prev] once, this moves forward.  
	///	- If you set a starting height, [Sub::set_block_height], it begins from there.  
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

	/// Returns the **previous block reference** `(hash, height)`.  
	///	- If you’ve already called [Sub::next] or [Sub::prev] once, this moves backwards.  
	/// - If you set a starting height, [Sub::set_block_height], it begins from `(height - 1)`.  
	/// - Otherwise, it starts from `(latest finalized/best height - 1)`.
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

	/// Returns `true` when RPC calls should be retried after failures.
	///
	/// The decision honors any explicit override configured via [`Sub::set_retry_on_error`]
	/// and falls back to the client's default retry policy when no override is provided.
	pub fn should_retry_on_error(&self) -> bool {
		let value = match self {
			Self::UnInit(u) => u.retry_on_error,
			Self::BestBlock(s) => s.retry_on_error,
			Self::FinalizedBlock(s) => s.retry_on_error,
		};

		should_retry(self.client_ref(), value)
	}

	/// Switches the subscription to track best (non-finalized) blocks when `value` is `true`.
	///
	/// This configuration must be applied before the subscription is initialized by a call to
	/// [`Sub::next`] or [`Sub::prev`]; later calls have no effect.
	pub fn use_best_block(&mut self, value: bool) {
		if let Self::UnInit(u) = self {
			u.use_best_block = value;
		}
	}

	/// Sets the initial block height used when iterating with [`Sub::next`] or [`Sub::prev`].
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

	/// Updates the polling interval used when waiting for new block data.
	pub fn set_pool_rate(&mut self, value: Duration) {
		match self {
			Self::UnInit(u) => u.poll_rate = value,
			Self::BestBlock(x) => x.poll_rate = value,
			Self::FinalizedBlock(x) => x.poll_rate = value,
		}
	}

	/// Overrides the retry policy for subsequent RPC calls made by the subscription.
	///
	/// Passing `Some(true)` forces retries, `Some(false)` disables them, and `None` restores
	/// the default behaviour derived from the [`Client`] configuration.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		match self {
			Self::UnInit(u) => u.retry_on_error = value,
			Self::BestBlock(x) => x.retry_on_error = value,
			Self::FinalizedBlock(x) => x.retry_on_error = value,
		}
	}

	pub(crate) fn client_ref(&self) -> &Client {
		match self {
			Sub::UnInit(x) => &x.client,
			Sub::BestBlock(x) => &x.client,
			Sub::FinalizedBlock(x) => &x.client,
		}
	}

	#[cfg(test)]
	pub(crate) fn as_best(&self) -> &BestBlockSub {
		if let Self::BestBlock(b) = self {
			return b;
		}
		panic!("Not best Sub");
	}

	#[cfg(test)]
	pub(crate) fn as_finalized(&self) -> &FinalizedBlockSub {
		if let Self::FinalizedBlock(f) = self {
			return f;
		}
		panic!("Not Finalized Sub");
	}
}

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
	pub(crate) next_block_height: u32,
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
	pub(crate) current_block_height: u32,
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{error::Error, prelude::*};

	#[tokio::test]
	pub async fn sub_test() -> Result<(), Error> {
		let client = Client::new(TURING_ENDPOINT).await?;
		let mut sub = Sub::new(client.clone());

		//
		//	Test Case 1: By default retires should be based around the global setting
		//
		client.set_global_retries_enabled(true);
		assert_eq!(sub.should_retry_on_error(), true);

		client.set_global_retries_enabled(false);
		assert_eq!(sub.should_retry_on_error(), false);

		//
		//	Test Case 2: Forcefully setting it to false should always yield false
		//
		sub.set_retry_on_error(Some(false));

		client.set_global_retries_enabled(true);
		assert_eq!(sub.should_retry_on_error(), false);

		client.set_global_retries_enabled(false);
		assert_eq!(sub.should_retry_on_error(), false);

		//
		//	Test Case 2: Forcefully setting it to true should always yield true
		//
		sub.set_retry_on_error(Some(true));

		client.set_global_retries_enabled(true);
		assert_eq!(sub.should_retry_on_error(), true);

		client.set_global_retries_enabled(false);
		assert_eq!(sub.should_retry_on_error(), true);

		Ok(())
	}

	// This test will be by flaky and that is OK.
	#[tokio::test]
	pub async fn best_sub_test() -> Result<(), Error> {
		let client = Client::new(TURING_ENDPOINT).await?;

		//
		// Test Case 1: Latest Block Height + Next
		//
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(true);

		let block_height = client.best().block_height().await?;
		let value = sub.next().await?;
		assert_eq!(value.height, block_height);

		//
		// Test Case 2: Latest Block Height + Prev
		//
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(true);

		let block_height = client.best().block_height().await?;
		let value = sub.prev().await?;
		assert_eq!(value.height, block_height - 1);

		//
		// Test Case 3: Set Block Height + Next + Next + Next
		//
		let block_height = 1900000u32;
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(true);
		sub.set_block_height(block_height);
		for i in 0..3 {
			let value = sub.next().await?;
			assert_eq!(value.height, block_height + i);
		}

		//
		// Test Case 4: Set Block Height + Prev + Prev + Prev
		//
		let block_height = 1900000u32;
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(true);
		sub.set_block_height(block_height);
		for i in 0..3 {
			let value = sub.prev().await?;
			assert_eq!(value.height, block_height - i - 1);
		}

		//
		// Test Case 5: Set Block Height + Next + Prev
		//
		let block_height = 1900000u32;
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(true);
		sub.set_block_height(block_height);

		let value = sub.next().await?;
		assert_eq!(value.height, block_height);

		let value = sub.prev().await?;
		assert_eq!(value.height, block_height - 1);

		//
		// Test Case 6: Set Block Height + Prev + Next
		//
		let block_height = 1900000u32;
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(true);
		sub.set_block_height(block_height);

		let value = sub.prev().await?;
		assert_eq!(value.height, block_height - 1);

		let value = sub.next().await?;
		assert_eq!(value.height, block_height);

		Ok(())
	}

	// This test will be by flaky and that is OK.
	#[tokio::test]
	pub async fn finalized_sub_test() -> Result<(), Error> {
		let client = Client::new(TURING_ENDPOINT).await?;

		//
		// Test Case 1: Latest Block Height + Next
		//
		let mut sub = Sub::new(client.clone());

		let block_height = client.finalized().block_height().await?;
		let value = sub.next().await?;
		assert_eq!(value.height, block_height);

		//
		// Test Case 2: Latest Block Height + Prev
		//
		let mut sub = Sub::new(client.clone());

		let block_height = client.finalized().block_height().await?;
		let value = sub.prev().await?;
		assert_eq!(value.height, block_height - 1);

		//
		// Test Case 3: Set Block Height + Next + Next + Next
		//
		let block_height = 1900000u32;
		let mut sub = Sub::new(client.clone());
		sub.set_block_height(block_height);
		for i in 0..3 {
			let value = sub.next().await?;
			assert_eq!(value.height, block_height + i);
		}

		//
		// Test Case 4: Set Block Height + Prev + Prev + Prev
		//
		let block_height = 1900000u32;
		let mut sub = Sub::new(client.clone());
		sub.set_block_height(block_height);
		for i in 0..3 {
			let value = sub.prev().await?;
			assert_eq!(value.height, block_height - i - 1);
		}

		//
		// Test Case 5: Set Block Height + Next + Prev
		//
		let block_height = 1900000u32;
		let mut sub = Sub::new(client.clone());
		sub.set_block_height(block_height);

		let value = sub.next().await?;
		assert_eq!(value.height, block_height);

		let value = sub.prev().await?;
		assert_eq!(value.height, block_height - 1);

		//
		// Test Case 6: Set Block Height + Prev + Next
		//
		let block_height = 1900000u32;
		let mut sub = Sub::new(client.clone());
		sub.set_block_height(block_height);

		let value = sub.prev().await?;
		assert_eq!(value.height, block_height - 1);

		let value = sub.next().await?;
		assert_eq!(value.height, block_height);

		Ok(())
	}
}
