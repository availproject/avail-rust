use crate::{Client, Error, block};
use avail_rust_core::{
	AccountIdLike, AvailHeader, BlockInfo, H256, RpcError,
	avail::{balances::types::AccountData, system::types::AccountInfo},
	rpc::LegacyBlock,
};

/// Helper bound to the chain's latest finalized block view.
pub struct Finalized {
	client: Client,
	retry_on_error: Option<bool>,
}

impl Finalized {
	/// Builds a helper focused on finalized blocks.
	pub fn new(client: Client) -> Self {
		Self { client, retry_on_error: None }
	}

	/// Lets you decide if upcoming calls retry on errors
	/// Overrides whether errors are retried (defaults to the client's global flag).
	pub fn retry_on(mut self, error: Option<bool>) -> Self {
		self.retry_on_error = error;
		self
	}

	/// Returns the hash of the latest finalized block.
	pub async fn block_hash(&self) -> Result<H256, RpcError> {
		self.block_info().await.map(|x| x.hash)
	}

	/// Returns the height of the latest finalized block.
	pub async fn block_height(&self) -> Result<u32, RpcError> {
		self.block_info().await.map(|x| x.height)
	}

	/// Returns the latest finalized block header.
	pub async fn block_header(&self) -> Result<AvailHeader, Error> {
		let block_hash = self.block_hash().await?;
		let block_header = self
			.client
			.chain()
			.retry_on(self.retry_on_error, Some(true))
			.block_header(Some(block_hash))
			.await?;
		let Some(block_header) = block_header else {
			return Err(RpcError::ExpectedData("Failed to fetch finalized block header".into()).into());
		};

		Ok(block_header)
	}

	/// Gives you a block handle for the latest finalized block.
	pub async fn block(&self) -> Result<block::Block, Error> {
		let block_hash = self.block_hash().await?;
		Ok(block::Block::new(self.client.clone(), block_hash))
	}

	/// Returns height and hash for the latest finalized block.
	///
	/// Equivalent to `chain().block_info(false)` with retry controls preserved.
	pub async fn block_info(&self) -> Result<BlockInfo, RpcError> {
		self.client
			.chain()
			.retry_on(self.retry_on_error, self.retry_on_error)
			.block_info(false)
			.await
	}

	/// Loads the legacy block view for the latest finalized block.
	///
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
			return Err(RpcError::ExpectedData("Failed to fetch finalized legacy block".into()));
		};

		Ok(block)
	}

	/// Returns the latest finalized nonce for the account.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		self.account_info(account_id).await.map(|v| v.nonce)
	}

	/// Returns account balances from the latest finalized block.
	pub async fn account_balance(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountData, Error> {
		self.account_info(account_id).await.map(|x| x.data)
	}

	/// Returns the full account record from the latest finalized block.
	///
	/// Errors mirror [`Chain::account_info`].
	pub async fn account_info(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountInfo, Error> {
		let at = self.block_hash().await?;
		self.client
			.chain()
			.retry_on(self.retry_on_error, Some(true))
			.account_info(account_id, at)
			.await
	}

	/// Returns true when finalized-block queries retry after RPC errors.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
	}
}
