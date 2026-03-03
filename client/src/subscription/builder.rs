use super::{
	fetcher::Fetcher,
	sub::{Sub, SubConfig},
	subscription::Subscription,
};
use crate::{Client, Error, RetryPolicy};
use std::time::Duration;

use super::sub::BlockQueryMode;

pub struct SubscriptionBuilder<F: Fetcher> {
	client: Client,
	fetcher: F,
	mode: BlockQueryMode,
	start_height: Option<u32>,
	poll_interval: Duration,
	retry_policy: RetryPolicy,
	skip_empty: bool,
}

impl<F: Fetcher> SubscriptionBuilder<F> {
	pub fn new(client: Client, fetcher: F) -> Self {
		Self {
			client,
			fetcher,
			mode: BlockQueryMode::Finalized,
			start_height: None,
			poll_interval: Duration::from_secs(3),
			retry_policy: RetryPolicy::Inherit,
			skip_empty: false,
		}
	}

	pub fn mode(mut self, mode: BlockQueryMode) -> Self {
		self.mode = mode;
		self
	}

	pub fn from_height(mut self, height: u32) -> Self {
		self.start_height = Some(height);
		self
	}

	pub fn poll_interval(mut self, interval: Duration) -> Self {
		self.poll_interval = interval;
		self
	}

	pub fn retry(mut self, policy: RetryPolicy) -> Self {
		self.retry_policy = policy;
		self
	}

	pub fn skip_empty(mut self) -> Self {
		self.skip_empty = true;
		self
	}

	pub async fn build(self) -> Result<Subscription<F>, Error> {
		let sub = self.init_sub().await?;
		Ok(Subscription { sub, fetcher: self.fetcher, skip_empty: self.skip_empty })
	}

	async fn init_sub(&self) -> Result<Sub, Error> {
		let config = SubConfig {
			mode: self.mode,
			start_height: self.start_height,
			poll_interval: self.poll_interval,
			retry_policy: self.retry_policy,
		};
		Sub::init(self.client.clone(), config).await.map_err(Error::from)
	}
}
