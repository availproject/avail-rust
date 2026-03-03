use crate::{Client, Error, RetryPolicy, avail, block, chain::Chain};
use avail::{balances::types::AccountData, system::types::AccountInfo};
use avail_rust_core::{
	AccountIdLike, AvailHeader, BlockInfo, H256,
	rpc::{Error as RpcError, LegacyBlock},
};

/// Selects which chain head to query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadKind {
	/// Follow the best (latest) head.
	Best,
	/// Follow the finalized head.
	Finalized,
}

/// Generic head helper used by [`crate::chain::Best`] and [`crate::chain::Finalized`].
pub struct Head {
	kind: HeadKind,
	chain: Chain,
}

impl Head {
	/// Creates a head helper for the requested head kind.
	pub fn new(client: Client, kind: HeadKind) -> Self {
		let chain = Chain::new(client).retry_policy(RetryPolicy::Inherit, RetryPolicy::Enabled);
		Self { kind, chain }
	}

	/// Sets retry behavior for subsequent head queries.
	pub fn retry_policy(mut self, policy: RetryPolicy) -> Self {
		self.chain = self.chain.retry_policy(policy, RetryPolicy::Enabled);
		self
	}

	pub async fn block_info(&self) -> Result<BlockInfo, RpcError> {
		let info = self.chain.info().await?;
		Ok(match self.kind {
			HeadKind::Best => BlockInfo { hash: info.best_hash, height: info.best_height },
			HeadKind::Finalized => BlockInfo { hash: info.finalized_hash, height: info.finalized_height },
		})
	}

	/// Returns the current head hash for this head kind.
	pub async fn block_hash(&self) -> Result<H256, RpcError> {
		let info = self.chain.info().await?;
		Ok(match self.kind {
			HeadKind::Best => info.best_hash,
			HeadKind::Finalized => info.finalized_hash,
		})
	}

	/// Returns the current head height for this head kind.
	pub async fn block_height(&self) -> Result<u32, RpcError> {
		let info = self.chain.info().await?;
		Ok(match self.kind {
			HeadKind::Best => info.best_height,
			HeadKind::Finalized => info.finalized_height,
		})
	}

	/// Returns the current head header.
	pub async fn block_header(&self) -> Result<AvailHeader, Error> {
		let block_hash = self.block_hash().await?;
		let block_header = self.chain.block_header(Some(block_hash)).await?;
		let Some(block_header) = block_header else {
			return Err(RpcError::ExpectedData("Failed to fetch head block header".into()).into());
		};

		Ok(block_header)
	}

	/// Returns a [`block::Block`] view for the current head.
	pub async fn block(&self) -> Result<block::Block, Error> {
		let block_hash = self.block_hash().await?;
		Ok(block::Block::new(self.chain.client.clone(), block_hash))
	}

	/// Returns the current head timestamp.
	pub async fn block_timestamp(&self) -> Result<u64, Error> {
		self.chain.block_timestamp(self.block_hash().await?).await
	}

	/// Returns the legacy block representation for the current head.
	pub async fn legacy_block(&self) -> Result<LegacyBlock, RpcError> {
		let block_hash = self.block_hash().await?;
		let block = self.chain.legacy_block(Some(block_hash)).await?;
		let Some(block) = block else {
			return Err(RpcError::ExpectedData("Failed to fetch latest legacy block".into()));
		};

		Ok(block)
	}

	/// Returns account nonce at the current head.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		self.account_info(account_id).await.map(|v| v.nonce)
	}

	/// Returns account balance at the current head.
	pub async fn account_balance(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountData, Error> {
		self.account_info(account_id).await.map(|x| x.data)
	}

	/// Returns account info at the current head.
	pub async fn account_info(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountInfo, Error> {
		let at = self.block_hash().await?;
		self.chain.account_info(account_id, at).await
	}

	/// Indicates whether head queries retry on failures.
	pub fn should_retry_on_error(&self) -> bool {
		self.chain.should_retry_on_error()
	}
}
