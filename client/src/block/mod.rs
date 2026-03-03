pub mod events;
pub mod extrinsic;
pub mod shared;

pub use events::{BlockEvent, BlockEvents, BlockEventsQuery};
pub use extrinsic::{BlockExtrinsicsQuery, TypedExtrinsic, UntypedExtrinsic};
pub use shared::BlockExtrinsicMetadata;

use crate::{Client, Error, RetryPolicy, block::shared::BlockContext};
use avail_rust_core::{
	AccountId, AccountIdLike, AvailHeader, BlockInfo, HashNumber,
	grandpa::GrandpaJustification,
	subxt_metadata,
	types::{
		HashStringNumber,
		substrate::{PerDispatchClassWeight, Weight},
	},
};

/// High-level handle bound to a specific block id (height or hash).
#[derive(Clone)]
pub struct Block {
	ctx: BlockContext,
}

impl Block {
	/// Constructs a block view from a hash or height.
	pub fn new(client: Client, at: impl Into<HashStringNumber>) -> Self {
		Block { ctx: BlockContext::new(client, at.into()) }
	}

	pub fn extrinsics(&self) -> extrinsic::BlockExtrinsicsQuery {
		extrinsic::BlockExtrinsicsQuery::new(self.ctx.client.clone(), self.ctx.block_id.clone())
	}

	/// Returns an event helper scoped to this block.
	pub fn events(&self) -> events::BlockEventsQuery {
		events::BlockEventsQuery::new(self.ctx.client.clone(), self.ctx.block_id.clone())
	}

	/// Sets retry behavior for subsequent block RPC calls.
	pub fn set_retry_policy(&mut self, value: RetryPolicy) {
		self.ctx.set_retry_policy(value);
	}

	/// Returns the GRANDPA justification for this block, if available.
	pub async fn justification(&self) -> Result<Option<GrandpaJustification>, Error> {
		let block_id: HashNumber = self.ctx.hash_number()?;
		let chain = self.ctx.chain();
		let at = match block_id {
			HashNumber::Hash(h) => chain.block_height(h).await?.ok_or_else(|| {
				Error::not_found_with_op(
					crate::error_ops::ErrorOperation::BlockJustification,
					"Failed to find block from provided hash",
				)
			})?,
			HashNumber::Number(n) => n,
		};

		chain.block_justification(at).await
	}

	/// Returns whether this helper retries RPC failures.
	pub fn should_retry_on_error(&self) -> bool {
		self.ctx.should_retry_on_error()
	}

	/// Returns this block's UNIX timestamp.
	pub async fn timestamp(&self) -> Result<u64, Error> {
		self.ctx.chain().block_timestamp(self.ctx.block_id.clone()).await
	}

	/// Returns block metadata (hash/height/parent data).
	pub async fn info(&self) -> Result<BlockInfo, Error> {
		self.ctx.chain().block_info_from(self.ctx.block_id.clone()).await
	}

	/// Returns this block header.
	pub async fn header(&self) -> Result<AvailHeader, Error> {
		self.ctx.header().await
	}

	/// Returns this block author.
	pub async fn author(&self) -> Result<AccountId, Error> {
		self.ctx.chain().block_author(self.ctx.block_id.clone()).await
	}

	/// Returns account nonce at this block.
	pub async fn nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		self.ctx
			.chain()
			.block_nonce(account_id, self.ctx.block_id.clone())
			.await
	}

	/// Returns the number of extrinsics in this block.
	pub async fn extrinsic_count(&self) -> Result<usize, Error> {
		let mut q = self.extrinsics();
		q.set_retry_policy(self.ctx.retry_policy());
		q.count(None, Default::default()).await
	}

	/// Returns the number of events in this block.
	pub async fn event_count(&self) -> Result<usize, Error> {
		self.ctx.event_count().await
	}

	/// Returns dispatch-class weight totals for this block.
	pub async fn weight(&self) -> Result<PerDispatchClassWeight, Error> {
		self.ctx.chain().block_weight(self.ctx.block_id.clone()).await
	}

	/// Returns total extrinsic weight inferred from block events.
	pub async fn extrinsic_weight(&self) -> Result<Weight, Error> {
		self.events().extrinsic_weight().await
	}

	/// TODO
	pub async fn metadata(&self) -> Result<subxt_metadata::Metadata, Error> {
		self.ctx.chain().block_metadata(Some(self.ctx.block_id.clone())).await
	}
}
