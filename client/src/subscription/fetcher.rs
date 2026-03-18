//! Fetcher trait and concrete implementations for each subscription data type.
//!
//! A [`Fetcher`] turns a [`BlockInfo`] reference into a domain-specific value
//! (header, legacy block, events, extrinsics, etc.) by issuing the appropriate
//! RPC calls against the connected node.

use crate::{
	AvailHeader, Client, Error, RetryPolicy,
	block::{self, Block, events::EventsQuery},
};
use async_trait::async_trait;
use avail_rust_core::{
	BlockInfo, HasHeader,
	grandpa::GrandpaJustification,
	rpc::{AllowedEvents, AllowedExtrinsic, LegacyBlock, PhaseEvents, SignatureFilter},
};
use codec::Decode;
use std::marker::PhantomData;

/// Transforms a block reference into a domain-specific value.
///
/// Every concrete fetcher implements this trait so that [`Subscription`](super::Subscription)
/// can be generic over the data it produces.
#[async_trait]
pub trait Fetcher: Clone {
	/// The value produced for each block.
	type Output;

	/// Fetches data for `info` using `client` and the resolved `retry` policy.
	async fn fetch(&self, client: &Client, info: BlockInfo, retry: RetryPolicy) -> Result<Self::Output, Error>;

	/// Returns `true` when `value` should be treated as "empty" for skip-empty logic.
	fn is_empty(&self, _value: &Self::Output) -> bool {
		false
	}
}

// ---------------------------------------------------------------------------
// 1. BlockInfo pass-through (the old `Sub`)
// ---------------------------------------------------------------------------

/// Yields raw [`BlockInfo`] without additional RPC calls.
#[derive(Debug, Clone, Copy)]
pub struct BlockInfoFetcher;

#[async_trait]
impl Fetcher for BlockInfoFetcher {
	type Output = ();

	async fn fetch(&self, _client: &Client, _info: BlockInfo, _retry: RetryPolicy) -> Result<(), Error> {
		Ok(())
	}
}

// ---------------------------------------------------------------------------
// 2. Block handle (lazy — no extra RPC at creation time)
// ---------------------------------------------------------------------------

/// Yields a lazy [`Block`] handle for each block.
#[derive(Debug, Clone, Copy)]
pub struct BlockFetcher;

#[async_trait]
impl Fetcher for BlockFetcher {
	type Output = Block;

	async fn fetch(&self, client: &Client, info: BlockInfo, _retry: RetryPolicy) -> Result<Block, Error> {
		Ok(Block::new(client.clone(), info.hash))
	}
}

// ---------------------------------------------------------------------------
// 3. Block header
// ---------------------------------------------------------------------------

/// Yields an [`AvailHeader`] for each block.
#[derive(Debug, Clone, Copy)]
pub struct BlockHeaderFetcher;

#[async_trait]
impl Fetcher for BlockHeaderFetcher {
	type Output = Option<AvailHeader>;

	async fn fetch(&self, client: &Client, info: BlockInfo, retry: RetryPolicy) -> Result<Option<AvailHeader>, Error> {
		client
			.chain()
			.retry_policy(retry, RetryPolicy::Enabled)
			.block_header(Some(info.hash))
			.await
	}
}

// ---------------------------------------------------------------------------
// 4. Legacy (signed) block
// ---------------------------------------------------------------------------

/// Yields a full [`LegacyBlock`] for each block.
#[derive(Debug, Clone, Copy)]
pub struct LegacyBlockFetcher;

#[async_trait]
impl Fetcher for LegacyBlockFetcher {
	type Output = Option<LegacyBlock>;

	async fn fetch(&self, client: &Client, info: BlockInfo, retry: RetryPolicy) -> Result<Option<LegacyBlock>, Error> {
		client
			.chain()
			.retry_policy(retry, RetryPolicy::Inherit)
			.legacy_block(Some(info.hash))
			.await
			.map_err(Error::from)
	}
}

// ---------------------------------------------------------------------------
// 5. Block events
// ---------------------------------------------------------------------------

/// Yields filtered event lists for each block.
#[derive(Debug, Clone)]
pub struct BlockEventsFetcher {
	pub(crate) allow_list: AllowedEvents,
}

#[async_trait]
impl Fetcher for BlockEventsFetcher {
	type Output = Vec<PhaseEvents>;

	async fn fetch(&self, client: &Client, info: BlockInfo, _retry: RetryPolicy) -> Result<Vec<PhaseEvents>, Error> {
		let query = EventsQuery::new(client.clone(), info.hash);
		query.rpc(self.allow_list.clone(), true).await
	}

	fn is_empty(&self, value: &Vec<PhaseEvents>) -> bool {
		value.is_empty()
	}
}

// ---------------------------------------------------------------------------
// 6. Typed (decoded) extrinsics
// ---------------------------------------------------------------------------

/// Yields decoded extrinsics of type `T` for each block.
#[derive(Clone)]
pub struct ExtrinsicFetcher<T: HasHeader + Decode> {
	pub(crate) sig_filter: SignatureFilter,
	pub(crate) _phantom: PhantomData<T>,
}

#[async_trait]
impl<T: HasHeader + Decode + Clone + Sync> Fetcher for ExtrinsicFetcher<T> {
	type Output = Vec<block::TypedExtrinsic<T>>;

	async fn fetch(&self, client: &Client, info: BlockInfo, retry: RetryPolicy) -> Result<Self::Output, Error> {
		let mut block = Block::new(client.clone(), info.hash).extrinsics();
		block.set_retry_policy(retry);
		block.all_as::<T>(self.sig_filter.clone()).await
	}

	fn is_empty(&self, value: &Self::Output) -> bool {
		value.is_empty()
	}
}

// ---------------------------------------------------------------------------
// 7. Untyped extrinsics
// ---------------------------------------------------------------------------

/// Yields raw, untyped extrinsic payloads for each block.
#[derive(Debug, Clone)]
pub struct UntypedExtrinsicFetcher {
	pub(crate) allow_list: Option<Vec<AllowedExtrinsic>>,
	pub(crate) sig_filter: SignatureFilter,
}

#[async_trait]
impl Fetcher for UntypedExtrinsicFetcher {
	type Output = Vec<block::UntypedExtrinsic>;

	async fn fetch(&self, client: &Client, info: BlockInfo, retry: RetryPolicy) -> Result<Self::Output, Error> {
		let mut block = Block::new(client.clone(), info.hash).extrinsics();
		block.set_retry_policy(retry);
		block.all(self.allow_list.clone(), self.sig_filter.clone()).await
	}

	fn is_empty(&self, value: &Self::Output) -> bool {
		value.is_empty()
	}
}

// ---------------------------------------------------------------------------
// 8. GRANDPA justification
// ---------------------------------------------------------------------------

/// Yields GRANDPA justifications for each block.
#[derive(Debug, Clone, Copy)]
pub struct GrandpaJustificationFetcher;

#[async_trait]
impl Fetcher for GrandpaJustificationFetcher {
	type Output = Option<GrandpaJustification>;

	async fn fetch(
		&self,
		client: &Client,
		info: BlockInfo,
		retry: RetryPolicy,
	) -> Result<Option<GrandpaJustification>, Error> {
		client
			.chain()
			.retry_policy(retry, RetryPolicy::Inherit)
			.block_justification(info.height)
			.await
	}
}
