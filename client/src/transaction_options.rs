//! Builders for configuring transaction submission defaults (nonce, tip, mortality).

use crate::{Client, subxt_core::config::Header};
use avail_rust_core::{AccountId, Era, ExtrinsicExtra, H256};

/// Lightweight builder for composing extrinsic signing options.
///
/// All fields default to `None`, deferring to runtime-derived values during [`Options::build`].
#[derive(Debug, Default, Clone, Copy)]
pub struct Options {
	pub app_id: Option<u32>,
	pub mortality: Option<MortalityOption>,
	pub nonce: Option<u32>,
	pub tip: Option<u128>,
}

impl Options {
	/// Starts a builder with the provided application id.
	pub fn new(app_id: u32) -> Self {
		Self { app_id: Some(app_id), ..Default::default() }
	}

	/// Sets the application id recorded in the extrinsic.
	pub fn app_id(mut self, value: u32) -> Self {
		self.app_id = Some(value);
		self
	}

	/// Sets the mortality configuration for the extrinsic.
	pub fn mortality(mut self, value: MortalityOption) -> Self {
		self.mortality = Some(value);
		self
	}

	/// Overrides the nonce to use when signing.
	pub fn nonce(mut self, value: u32) -> Self {
		self.nonce = Some(value);
		self
	}

	/// Overrides the tip to attach to the extrinsic.
	pub fn tip(mut self, value: u128) -> Self {
		self.tip = Some(value);
		self
	}

	/// Resolves all builder values into concrete options ready for signing.
	///
	/// # Returns
	/// - `Ok(RefinedOptions)` containing explicit nonce, tip, app id, and mortality details.
	/// - `Err(crate::Error)` when fetching account information or finality data fails.
	///
	/// # Behaviour
	/// - Missing nonce triggers an RPC call to fetch the account's next nonce.
	/// - Missing mortality defaults to a 32-block period anchored at the latest finalized block.
	/// - Missing app id and tip default to zero.
	pub async fn build(
		self,
		client: &Client,
		account_id: &AccountId,
		retry_on_error: Option<bool>,
	) -> Result<RefinedOptions, crate::Error> {
		let app_id = self.app_id.unwrap_or_default();
		let tip = self.tip.unwrap_or_default();
		let nonce = match self.nonce {
			Some(x) => x,
			None => {
				client
					.chain()
					.retry_on(retry_on_error, None)
					.account_nonce(account_id.clone())
					.await?
			},
		};
		let mortality = self.mortality.unwrap_or(MortalityOption::Period(32));
		let mortality = match mortality {
			MortalityOption::Period(period) => RefinedMortality::from_period(client, period).await?,
			MortalityOption::Full(mortality) => mortality,
		};

		Ok(RefinedOptions { app_id, mortality, nonce, tip })
	}
}

/// Fully resolved transaction options used during signing.
#[derive(Debug, Clone)]
pub struct RefinedOptions {
	pub app_id: u32,
	pub mortality: RefinedMortality,
	pub nonce: u32,
	pub tip: u128,
}

impl From<&RefinedOptions> for ExtrinsicExtra {
	fn from(value: &RefinedOptions) -> Self {
		let era = Era::mortal(value.mortality.period, value.mortality.block_height as u64);
		ExtrinsicExtra {
			era,
			nonce: value.nonce,
			tip: value.tip,
			app_id: value.app_id,
		}
	}
}

/// User-facing mortality configuration options.
#[derive(Debug, Clone, Copy)]
pub enum MortalityOption {
	/// Mortality based on a relative period (number of blocks) anchored at the finalized head.
	Period(u64),
	/// Fully specified mortality with explicit block hash and height.
	Full(RefinedMortality),
}

/// Mortality with resolved block hash/height anchors.
#[derive(Debug, Clone, Copy)]
pub struct RefinedMortality {
	pub period: u64,
	pub block_hash: H256,
	pub block_height: u32,
}
impl RefinedMortality {
	/// Creates a refined mortality value.
	pub fn new(period: u64, block_hash: H256, block_height: u32) -> Self {
		Self { period, block_hash, block_height }
	}

	/// Derives mortality from the latest finalized header using the given period.
	///
	/// # Errors
	/// Returns `Err(crate::Error)` when fetching the finalized header fails.
	pub async fn from_period(client: &Client, period: u64) -> Result<Self, crate::Error> {
		let header = client.finalized().block_header().await?;
		let (block_hash, block_height) = (header.hash(), header.number());
		Ok(Self { period, block_hash, block_height })
	}
}
