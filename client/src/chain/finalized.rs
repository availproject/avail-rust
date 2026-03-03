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

/// Convenience helper for querying the finalized head.
pub struct Finalized {
	head: Head,
}

impl Finalized {
	/// Creates a finalized-head helper.
	pub fn new(client: Client) -> Self {
		Self { head: Head::new(client, HeadKind::Finalized) }
	}

	/// Sets retry behavior for subsequent finalized-head queries.
	pub fn retry_policy(mut self, policy: RetryPolicy) -> Self {
		self.head = self.head.retry_policy(policy);
		self
	}

	/// TODO
	pub async fn block_info(&self) -> Result<BlockInfo, RpcError> {
		self.head.block_info().await
	}

	/// Returns the current finalized block hash.
	pub async fn block_hash(&self) -> Result<H256, RpcError> {
		self.head.block_hash().await
	}

	/// Returns the current finalized block height.
	pub async fn block_height(&self) -> Result<u32, RpcError> {
		self.head.block_height().await
	}

	/// Returns the current finalized block header.
	pub async fn block_header(&self) -> Result<AvailHeader, Error> {
		self.head.block_header().await
	}

	/// Returns a block view for the current finalized block.
	pub async fn block(&self) -> Result<block::Block, Error> {
		self.head.block().await
	}

	/// Returns the timestamp of the current finalized block.
	pub async fn block_timestamp(&self) -> Result<u64, Error> {
		self.head.block_timestamp().await
	}

	/// Returns the current finalized block in legacy representation.
	pub async fn signed(&self) -> Result<LegacyBlock, RpcError> {
		self.head.legacy_block().await
	}

	/// Alias for [`Finalized::signed`].
	pub async fn legacy_block(&self) -> Result<LegacyBlock, RpcError> {
		self.signed().await
	}

	/// Returns account nonce at the current finalized block.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		self.head.account_nonce(account_id).await
	}

	/// Returns account balance at the current finalized block.
	pub async fn account_balance(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountData, Error> {
		self.head.account_balance(account_id).await
	}

	/// Returns account info at the current finalized block.
	pub async fn account_info(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountInfo, Error> {
		self.head.account_info(account_id).await
	}

	/// Indicates whether finalized-head queries retry on failures.
	pub fn should_retry_on_error(&self) -> bool {
		self.head.should_retry_on_error()
	}
}
