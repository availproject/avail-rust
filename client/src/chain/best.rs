use crate::{
	Client, Error, RetryPolicy,
	avail::balances::types::AccountData,
	block,
	chain::{Head, HeadKind},
};
use avail_rust_core::{
	AccountIdLike, AvailHeader, BlockInfo, H256,
	avail::system::types::AccountInfo,
	rpc::{Error as RpcError, LegacyBlock},
};

/// Convenience helper for querying the best (latest) head.
pub struct Best {
	head: Head,
}

impl Best {
	/// Creates a best-head helper.
	pub fn new(client: Client) -> Self {
		Self { head: Head::new(client, HeadKind::Best) }
	}

	/// Sets retry behavior for subsequent best-head queries.
	pub fn retry_policy(mut self, policy: RetryPolicy) -> Self {
		self.head = self.head.retry_policy(policy);
		self
	}

	/// TODO
	pub async fn block_info(&self) -> Result<BlockInfo, RpcError> {
		self.head.block_info().await
	}

	/// Returns the current best block hash.
	pub async fn block_hash(&self) -> Result<H256, RpcError> {
		self.head.block_hash().await
	}

	/// Returns the current best block height.
	pub async fn block_height(&self) -> Result<u32, RpcError> {
		self.head.block_height().await
	}

	/// Returns the current best block header.
	pub async fn block_header(&self) -> Result<AvailHeader, Error> {
		self.head.block_header().await
	}

	/// Returns a block view for the current best block.
	pub async fn block(&self) -> Result<block::Block, Error> {
		self.head.block().await
	}

	/// Returns the timestamp of the current best block.
	pub async fn block_timestamp(&self) -> Result<u64, Error> {
		self.head.block_timestamp().await
	}

	/// Alias for [`Best::signed`].
	pub async fn legacy_block(&self) -> Result<LegacyBlock, RpcError> {
		self.head.legacy_block().await
	}

	/// Returns account nonce at the current best block.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		self.head.account_nonce(account_id).await
	}

	/// Returns account balance at the current best block.
	pub async fn account_balance(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountData, Error> {
		self.head.account_balance(account_id).await
	}

	/// Returns account info at the current best block.
	pub async fn account_info(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountInfo, Error> {
		self.head.account_info(account_id).await
	}

	/// Indicates whether best-head queries retry on failures.
	pub fn should_retry_on_error(&self) -> bool {
		self.head.should_retry_on_error()
	}
}
