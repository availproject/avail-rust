use super::{fetcher::Fetcher, sub::Sub};
use crate::Error;
use avail_rust_core::{BlockInfo, H256};
use futures::stream::{self, Stream};

#[derive(Debug, Clone)]
pub struct SubscriptionItem<T> {
	pub value: T,
	pub block_height: u32,
	pub block_hash: H256,
}

pub struct Subscription<F: Fetcher> {
	pub(super) sub: Sub,
	pub(super) fetcher: F,
	pub(super) skip_empty: bool,
}

impl<F: Fetcher> Subscription<F> {
	pub async fn next(&mut self) -> Result<SubscriptionItem<F::Output>, Error> {
		loop {
			let info = self.sub.next().await?;
			match self.fetch_at(info).await {
				Ok(Some(item)) => return Ok(item),
				Ok(None) => continue,
				Err(e) => return Err(e),
			}
		}
	}

	pub async fn prev(&mut self) -> Result<SubscriptionItem<F::Output>, Error> {
		loop {
			let info = self.sub.prev().await?;
			match self.fetch_at(info).await {
				Ok(Some(item)) => return Ok(item),
				Ok(None) => continue,
				Err(e) => return Err(e),
			}
		}
	}

	pub fn into_stream(self) -> impl Stream<Item = Result<SubscriptionItem<F::Output>, Error>> {
		stream::try_unfold(self, |mut this| async move {
			let item = this.next().await?;
			Ok(Some((item, this)))
		})
	}

	async fn fetch_at(&mut self, info: BlockInfo) -> Result<Option<SubscriptionItem<F::Output>>, Error> {
		let client = self.sub.client_ref().clone();
		let retry = self.sub.resolved_retry_policy();

		match self.fetcher.fetch(&client, info, retry).await {
			Ok(value) => {
				if self.skip_empty && self.fetcher.is_empty(&value) {
					return Ok(None);
				}
				Ok(Some(SubscriptionItem { value, block_height: info.height, block_hash: info.hash }))
			},
			Err(e) => {
				self.sub.set_block_height(info.height);
				Err(e)
			},
		}
	}
}
