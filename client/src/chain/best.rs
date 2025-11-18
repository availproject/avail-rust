use crate::{Client, Error, avail, block, chain::Chain};
use avail::{balances::types::AccountData, system::types::AccountInfo};
use avail_rust_core::{
	AccountIdLike, AvailHeader, BlockInfo, H256,
	rpc::{Error as RpcError, LegacyBlock},
};

/// Helper bound to the chain's best (head) block view.
pub struct Best {
	chain: Chain,
}
impl Best {
	/// Builds a helper focused on the best (head) block.
	///
	/// # Arguments
	/// * `client` - Client used to perform RPC calls.
	///
	/// # Returns
	/// Returns a [`Best`] helper that inherits the client's retry settings.
	pub fn new(client: Client) -> Self {
		let chain = Chain::new(client).retry_on(None, Some(true));
		Self { chain }
	}

	/// Overrides whether upcoming calls retry after RPC errors.
	///
	/// # Arguments
	/// * `error` - `Some(true)` to force retries, `Some(false)` to disable retries, `None` to inherit defaults.
	///
	/// # Returns
	/// Returns the helper with the updated retry preference.
	pub fn retry_on(mut self, error: Option<bool>) -> Self {
		self.chain = self.chain.retry_on(error, Some(true));
		self
	}

	/// Returns the hash of the best block.
	///
	/// # Returns
	/// Returns the hash of the current head block.
	///
	/// # Errors
	/// Propagates any RPC error encountered while fetching block information.
	pub async fn block_hash(&self) -> Result<H256, RpcError> {
		self.block_info().await.map(|x| x.hash)
	}

	/// Returns the height of the best block.
	///
	/// # Returns
	/// Returns the block number recorded for the chain head.
	///
	/// # Errors
	/// Propagates any RPC error encountered while fetching block information.
	pub async fn block_height(&self) -> Result<u32, RpcError> {
		self.block_info().await.map(|x| x.height)
	}

	/// Returns the current best block header.
	///
	/// # Returns
	/// Returns the header associated with the current head block.
	///
	/// # Errors
	/// Returns `Err(Error)` when the node does not provide a header or the RPC call fails.
	pub async fn block_header(&self) -> Result<AvailHeader, Error> {
		let block_hash = self.block_hash().await?;
		let block_header = self.chain.block_header(Some(block_hash)).await?;
		let Some(block_header) = block_header else {
			return Err(RpcError::ExpectedData("Failed to fetch best block header".into()).into());
		};

		Ok(block_header)
	}

	/// Returns a block helper bound to the current head block.
	///
	/// # Returns
	/// Returns a [`block::Block`] helper scoped to the head block.
	///
	/// # Errors
	/// Returns `Err(Error)` when the head hash cannot be fetched.
	pub async fn block(&self) -> Result<block::Block, Error> {
		let block_hash = self.block_hash().await?;
		Ok(block::Block::new(self.chain.client.clone(), block_hash))
	}

	/// Returns height and hash for the best block.
	///
	/// # Returns
	/// Returns [`BlockInfo`] describing the newest block.
	///
	/// # Errors
	/// Propagates any RPC error encountered while fetching block information.
	pub async fn block_info(&self) -> Result<BlockInfo, RpcError> {
		self.chain.block_info(true).await
	}

	/// Loads the legacy block for the current head.
	///
	/// # Returns
	/// Returns the legacy block representation.
	///
	/// # Errors
	/// Returns `Err(RpcError::ExpectedData)` when the node reports no legacy block for the head.
	pub async fn legacy_block(&self) -> Result<LegacyBlock, RpcError> {
		let block_hash = self.block_hash().await?;
		let block = self.chain.legacy_block(Some(block_hash)).await?;
		let Some(block) = block else {
			return Err(RpcError::ExpectedData("Failed to fetch latest legacy block".into()));
		};

		Ok(block)
	}

	/// Returns the latest nonce for the account at the best block.
	///
	/// # Arguments
	/// * `account_id` - Account identifier convertible into `AccountIdLike`.
	///
	/// # Returns
	/// Returns the account nonce observed at the head block.
	///
	/// # Errors
	/// Returns `Err(Error)` when the account cannot be parsed or the RPC call fails.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		self.account_info(account_id).await.map(|v| v.nonce)
	}

	/// Returns the account balances at the best block.
	///
	/// # Arguments
	/// * `account_id` - Account identifier convertible into `AccountIdLike`.
	///
	/// # Returns
	/// Returns the account data (free, reserved, frozen balances) observed at the head block.
	///
	/// # Errors
	/// Returns `Err(Error)` when the account cannot be parsed or the RPC call fails.
	pub async fn account_balance(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountData, Error> {
		self.account_info(account_id).await.map(|x| x.data)
	}

	/// Returns the full account record at the best block.
	///
	/// # Arguments
	/// * `account_id` - Account identifier convertible into `AccountIdLike`.
	///
	/// # Returns
	/// Returns [`AccountInfo`] describing balances, consumers, and nonce at the head block.
	///
	/// # Errors
	/// Returns `Err(Error)` when the account cannot be parsed or the RPC call fails.
	pub async fn account_info(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountInfo, Error> {
		let at = self.block_hash().await?;
		self.chain.account_info(account_id, at).await
	}

	/// Reports whether best-block queries retry after RPC errors.
	///
	/// # Returns
	/// Returns `true` when retries are enabled, otherwise `false`.
	pub fn should_retry_on_error(&self) -> bool {
		self.chain.should_retry_on_error()
	}
}
