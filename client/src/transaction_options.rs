use crate::{Client, subxt_core::config::Header};
use avail_rust_core::{AccountId, Era, H256, TransactionExtra};

#[derive(Debug, Default, Clone, Copy)]
pub struct Options {
	pub app_id: Option<u32>,
	pub mortality: Option<MortalityOption>,
	pub nonce: Option<u32>,
	pub tip: Option<u128>,
}

impl Options {
	pub fn new(app_id: Option<u32>) -> Self {
		let mut s = Self::default();
		s.app_id = app_id;
		s
	}

	pub fn app_id(mut self, value: u32) -> Self {
		self.app_id = Some(value);
		self
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

	pub async fn build(
		self,
		client: &Client,
		account_id: &AccountId,
	) -> Result<RefinedOptions, avail_rust_core::Error> {
		let app_id = self.app_id.unwrap_or_default();
		let tip = self.tip.unwrap_or_default();
		let nonce = match self.nonce {
			Some(x) => x,
			None => {
				client
					.rpc_api()
					.system_account_next_index(&account_id.to_string())
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

#[derive(Debug, Clone)]
pub struct RefinedOptions {
	pub app_id: u32,
	pub mortality: RefinedMortality,
	pub nonce: u32,
	pub tip: u128,
}

impl From<&RefinedOptions> for TransactionExtra {
	fn from(value: &RefinedOptions) -> Self {
		let era = Era::mortal(value.mortality.period, value.mortality.block_height as u64);
		TransactionExtra {
			era,
			nonce: value.nonce,
			tip: value.tip,
			app_id: value.app_id,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum MortalityOption {
	Period(u64),
	Full(RefinedMortality),
}

#[derive(Debug, Clone, Copy)]
pub struct RefinedMortality {
	pub period: u64,
	pub block_hash: H256,
	pub block_height: u32,
}
impl RefinedMortality {
	pub fn new(period: u64, block_hash: H256, block_height: u32) -> Self {
		Self { period, block_hash, block_height }
	}

	pub async fn from_period(client: &Client, period: u64) -> Result<Self, avail_rust_core::Error> {
		let header = client.finalized_block_header().await?;
		let (block_hash, block_height) = (header.hash(), header.number());
		Ok(Self { period, block_hash, block_height })
	}
}
