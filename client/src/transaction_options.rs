//! Builders for configuring transaction submission defaults (nonce, tip, mortality).

use crate::{Client, RetryPolicy};
use avail_rust_core::{AccountId, Era, Extension, H256};

#[derive(Debug, Default, Clone, Copy)]
pub struct Options {
	pub mortality: Option<MortalityOption>,
	pub nonce: Option<u32>,
	pub tip: Option<u128>,
}

impl Options {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn mortality(mut self, value: MortalityOption) -> Self {
		self.mortality = Some(value);
		self
	}

	pub fn nonce(mut self, value: u32) -> Self {
		self.nonce = Some(value);
		self
	}

	pub fn tip(mut self, value: u128) -> Self {
		self.tip = Some(value);
		self
	}

	pub async fn resolve_nonce(
		self,
		client: &Client,
		account_id: &AccountId,
		retry_on_error: RetryPolicy,
	) -> Result<u32, crate::Error> {
		let nonce = match self.nonce {
			Some(x) => x,
			None => {
				client
					.chain()
					.retry_policy(retry_on_error, RetryPolicy::Inherit)
					.account_nonce(account_id.clone())
					.await?
			},
		};

		Ok(nonce)
	}

	pub async fn resolve_mortality(self, client: &Client) -> Result<Mortality, crate::Error> {
		let mortality = self.mortality.unwrap_or(MortalityOption::Period(32));
		let mortality = match mortality {
			MortalityOption::Period(period) => Mortality::from_period(client, period).await?,
			MortalityOption::Full(mortality) => mortality,
		};

		Ok(mortality)
	}

	pub async fn resolve(
		self,
		client: &Client,
		account_id: &AccountId,
		retry_on_error: RetryPolicy,
	) -> Result<ResolvedOptions, crate::Error> {
		let tip = self.tip.unwrap_or_default();
		let nonce = self.resolve_nonce(client, account_id, retry_on_error).await?;
		let mortality = self.resolve_mortality(client).await?;

		Ok(ResolvedOptions { mortality, nonce, tip })
	}
}

#[derive(Debug, Clone)]
pub struct ResolvedOptions {
	pub mortality: Mortality,
	pub nonce: u32,
	pub tip: u128,
}

impl From<&ResolvedOptions> for Extension {
	fn from(value: &ResolvedOptions) -> Self {
		let era = Era::mortal(value.mortality.period, value.mortality.block_height as u64);
		Extension { era, nonce: value.nonce, tip: value.tip }
	}
}

#[derive(Debug, Clone, Copy)]
pub enum MortalityOption {
	Period(u64),
	Full(Mortality),
}

#[derive(Debug, Clone, Copy)]
pub struct Mortality {
	pub period: u64,
	pub block_hash: H256,
	pub block_height: u32,
}

impl Mortality {
	pub fn new(period: u64, block_hash: H256, block_height: u32) -> Self {
		Self { period, block_hash, block_height }
	}

	pub async fn from_period(client: &Client, period: u64) -> Result<Self, crate::Error> {
		let info = client.chain().info().await?;
		let (block_hash, block_height) = (info.finalized_hash, info.finalized_height);
		Ok(Self { period, block_hash, block_height })
	}
}
