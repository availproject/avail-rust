//! Builders for configuring transaction submission defaults (nonce, tip, mortality).

use crate::{Client, subxt_core::config::Header};
use avail_rust_core::{AccountId, Era, ExtrinsicExtra, H256};

/// Lightweight builder for composing extrinsic signing options.
///
/// All fields default to `None`, deferring to runtime-derived values during [`Options::build`].
#[derive(Debug, Default, Clone, Copy)]
pub struct Options {
	/// Application identifier recorded in the signature payload.
	pub app_id: Option<u32>,
	/// Mortality configuration prior to refinement.
	pub mortality: Option<MortalityOption>,
	/// Nonce override to use during signing.
	pub nonce: Option<u32>,
	/// Tip (in smallest units) to attach to the extrinsic.
	pub tip: Option<u128>,
}

impl Options {
	/// Starts a builder with the provided application id.
	///
	/// # Arguments
	/// * `app_id` - Application identifier recorded in the signature payload.
	///
	/// # Returns
	/// Returns an [`Options`] builder seeded with the supplied application id.
	pub fn new(app_id: u32) -> Self {
		Self { app_id: Some(app_id), ..Default::default() }
	}

	/// Sets the application id recorded in the extrinsic.
	///
	/// # Arguments
	/// * `value` - Application identifier to embed in the extrinsic.
	///
	/// # Returns
	/// Returns the builder with the application id updated.
	pub fn app_id(mut self, value: u32) -> Self {
		self.app_id = Some(value);
		self
	}

	/// Sets the mortality configuration for the extrinsic.
	///
	/// # Arguments
	/// * `value` - Mortality option describing how long the extrinsic remains valid.
	///
	/// # Returns
	/// Returns the builder with the mortality updated.
	pub fn mortality(mut self, value: MortalityOption) -> Self {
		self.mortality = Some(value);
		self
	}

	/// Overrides the nonce to use when signing.
	///
	/// # Arguments
	/// * `value` - Nonce that should be used when constructing the payload.
	///
	/// # Returns
	/// Returns the builder with the nonce override applied.
	pub fn nonce(mut self, value: u32) -> Self {
		self.nonce = Some(value);
		self
	}

	/// Overrides the tip to attach to the extrinsic.
	///
	/// # Arguments
	/// * `value` - Tip (in smallest units) applied to the extrinsic.
	///
	/// # Returns
	/// Returns the builder with the tip override applied.
	pub fn tip(mut self, value: u128) -> Self {
		self.tip = Some(value);
		self
	}

	/// Resolves all builder values into concrete options ready for signing.
	///
	/// # Arguments
	/// * `client` - Client used to fetch on-chain data when defaults are missing.
	/// * `account_id` - Account whose nonce and mortality anchor are derived.
	/// * `retry_on_error` - Optional override controlling retry behaviour for RPC calls.
	///
	/// # Returns
	/// - `Ok(RefinedOptions)` containing explicit nonce, tip, app id, and mortality details.
	/// - `Err(crate::Error)` when fetching account information or finality data fails.
	///
	/// # Errors
	/// Returns `Err(crate::Error)` when RPC lookups required to refine options fail.
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
	/// Application identifier recorded in the extrinsic.
	pub app_id: u32,
	/// Fully resolved mortality parameters.
	pub mortality: RefinedMortality,
	/// Nonce applied to the extrinsic.
	pub nonce: u32,
	/// Tip (in smallest units) attached to the extrinsic.
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
	/// Number of blocks before the extrinsic becomes invalid.
	pub period: u64,
	/// Block hash anchoring the mortality.
	pub block_hash: H256,
	/// Block height anchoring the mortality.
	pub block_height: u32,
}
impl RefinedMortality {
	/// Creates a refined mortality value.
	///
	/// # Arguments
	/// * `period` - Number of blocks after which the extrinsic expires.
	/// * `block_hash` - Block hash anchoring the mortality.
	/// * `block_height` - Height corresponding to `block_hash`.
	///
	/// # Returns
	/// Returns a [`RefinedMortality`] struct encapsulating the supplied values.
	pub fn new(period: u64, block_hash: H256, block_height: u32) -> Self {
		Self { period, block_hash, block_height }
	}

	/// Derives mortality from the latest finalized header using the given period.
	///
	/// # Arguments
	/// * `client` - Client used to fetch the latest finalised header.
	/// * `period` - Number of blocks after which the extrinsic expires.
	///
	/// # Errors
	/// Returns `Err(crate::Error)` when fetching the finalized header fails.
	pub async fn from_period(client: &Client, period: u64) -> Result<Self, crate::Error> {
		let header = client.finalized().block_header().await?;
		let (block_hash, block_height) = (header.hash(), header.number());
		Ok(Self { period, block_hash, block_height })
	}
}
