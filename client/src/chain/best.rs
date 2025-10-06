use crate::{Client, Error, avail, block_api::BlockApi};
use avail::{balances::types::AccountData, system::types::AccountInfo};
use avail_rust_core::{
	AccountIdLike, AvailHeader, BlockInfo, H256,
	rpc::{Error as RpcError, LegacyBlock},
};

/// Helper bound to the chain's best (head) block view.
pub struct Best {
	client: Client,
	retry_on_error: Option<bool>,
}
impl Best {
	/// Builds a helper focused on the best (head) block.
	pub fn new(client: Client) -> Self {
		Self { client, retry_on_error: None }
	}

	/// Lets you decide if upcoming calls retry on errors
	/// Overrides whether errors are retried (defaults to the client's global flag).
	pub fn retry_on(mut self, error: Option<bool>) -> Self {
		self.retry_on_error = error;
		self
	}

	/// Returns the hash of the best block.
	pub async fn block_hash(&self) -> Result<H256, RpcError> {
		self.block_info().await.map(|x| x.hash)
	}

	/// Returns the height of the best block.
	pub async fn block_height(&self) -> Result<u32, RpcError> {
		self.block_info().await.map(|x| x.height)
	}

	/// Returns the current best block header.
	pub async fn block_header(&self) -> Result<AvailHeader, Error> {
		let block_hash = self.block_hash().await?;
		let block_header = self
			.client
			.chain()
			.retry_on(self.retry_on_error, Some(true))
			.block_header(Some(block_hash))
			.await?;
		let Some(block_header) = block_header else {
			return Err(RpcError::ExpectedData("Failed to fetch best block header".into()).into());
		};

		Ok(block_header)
	}

	/// Gives you a block handle for the best block.
	pub async fn block(&self) -> Result<BlockApi, Error> {
		let block_hash = self.block_hash().await?;
		Ok(BlockApi::new(self.client.clone(), block_hash))
	}

	/// Returns height and hash for the best block.
	///
	/// Equivalent to `chain().block_info(true)` but respecting this helper's retry setting.
	pub async fn block_info(&self) -> Result<BlockInfo, RpcError> {
		self.client
			.chain()
			.retry_on(self.retry_on_error, Some(true))
			.block_info(true)
			.await
	}

	/// Loads the legacy block for the best block.
	///
	/// # Errors
	/// Returns `Err(RpcError::ExpectedData)` when the node reports no legacy block for the head.
	pub async fn legacy_block(&self) -> Result<LegacyBlock, RpcError> {
		let block_hash = self.block_hash().await?;
		let block = self
			.client
			.chain()
			.retry_on(self.retry_on_error, Some(true))
			.legacy_block(Some(block_hash))
			.await?;
		let Some(block) = block else {
			return Err(RpcError::ExpectedData("Failed to fetch latest legacy block".into()));
		};

		Ok(block)
	}

	/// Returns the latest nonce for the account at the best block.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		self.account_info(account_id).await.map(|v| v.nonce)
	}

	/// Returns the account balances at the best block.
	pub async fn account_balance(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountData, Error> {
		self.account_info(account_id).await.map(|x| x.data)
	}

	/// Returns the full account record at the best block.
	pub async fn account_info(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountInfo, Error> {
		let at = self.block_hash().await?;
		self.client
			.chain()
			.retry_on(self.retry_on_error, Some(true))
			.account_info(account_id, at)
			.await
	}

	/// Returns true when best-block queries retry after RPC errors.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
	}
}
