pub mod builder;
pub mod fetcher;
pub mod sub;

pub use builder::SubscriptionBuilder;
pub use fetcher::{
	BlockEventsFetcher, BlockFetcher, BlockHeaderFetcher, BlockInfoFetcher, ExtrinsicFetcher, Fetcher,
	GrandpaJustificationFetcher, LegacyBlockFetcher, UntypedExtrinsicFetcher,
};
pub use sub::{BlockQueryMode, Subscription, SubscriptionItem};

use crate::Client;
use avail_rust_core::{
	HasHeader,
	rpc::{AllowedEvents, AllowedExtrinsic, Query},
};
use codec::Decode;
use std::marker::PhantomData;

pub struct SubscribeApi(pub(crate) Client);

impl SubscribeApi {
	pub fn raw(&self) -> SubscriptionBuilder<BlockInfoFetcher> {
		SubscriptionBuilder::new(self.0.clone(), BlockInfoFetcher)
	}

	pub fn blocks(&self) -> SubscriptionBuilder<BlockFetcher> {
		SubscriptionBuilder::new(self.0.clone(), BlockFetcher)
	}

	pub fn block_headers(&self) -> SubscriptionBuilder<BlockHeaderFetcher> {
		SubscriptionBuilder::new(self.0.clone(), BlockHeaderFetcher)
	}

	pub fn block_events(&self, allow_list: AllowedEvents) -> SubscriptionBuilder<BlockEventsFetcher> {
		SubscriptionBuilder::new(self.0.clone(), BlockEventsFetcher { allow_list })
	}

	pub fn legacy_blocks(&self) -> SubscriptionBuilder<LegacyBlockFetcher> {
		SubscriptionBuilder::new(self.0.clone(), LegacyBlockFetcher)
	}

	pub fn extrinsics<T: HasHeader + Decode + Clone + Sync>(
		&self,
		query: Query,
	) -> SubscriptionBuilder<ExtrinsicFetcher<T>> {
		SubscriptionBuilder::new(self.0.clone(), ExtrinsicFetcher { query, _phantom: PhantomData })
	}

	pub fn untyped_extrinsics(
		&self,
		allow_list: Option<Vec<AllowedExtrinsic>>,
		query: Query,
	) -> SubscriptionBuilder<UntypedExtrinsicFetcher> {
		SubscriptionBuilder::new(self.0.clone(), UntypedExtrinsicFetcher { allow_list, query })
	}

	pub fn justification(&self) -> SubscriptionBuilder<GrandpaJustificationFetcher> {
		SubscriptionBuilder::new(self.0.clone(), GrandpaJustificationFetcher)
	}
}
