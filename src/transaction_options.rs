use crate::{client::Client, config::*, error::RpcError};
use primitive_types::H256;
use subxt::config::Header;

pub type Params =
	<<AvailConfig as subxt::Config>::ExtrinsicParams as subxt::config::ExtrinsicParams<AvailConfig>>::Params;

#[derive(Debug, Clone, Copy)]
pub struct Options {
	pub app_id: Option<u32>,
	pub mortality: Option<MortalityOption>,
	pub nonce: Option<u32>,
	pub tip: Option<u128>,
}

impl Options {
	pub fn new() -> Self {
		Self {
			app_id: None,
			mortality: None,
			nonce: None,
			tip: None,
		}
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

	pub async fn build(self, client: &Client, account_id: &AccountId) -> Result<PopulatedOptions, RpcError> {
		let app_id = self.app_id.unwrap_or_default();
		let tip = self.tip.unwrap_or_default();
		let nonce = match self.nonce {
			Some(x) => x as u64,
			None => client.rpc_system_account_next_index(&account_id.to_string()).await? as u64,
		};
		let mortality = match &self.mortality {
			Some(MortalityOption::Period(period)) => Mortality::from_period(client, *period).await?,
			Some(MortalityOption::Full(mortality)) => *mortality,
			None => Mortality::from_period(client, 32).await?,
		};

		Ok(PopulatedOptions {
			app_id,
			mortality,
			nonce,
			tip,
		})
	}
}

impl Default for Options {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone)]
pub struct PopulatedOptions {
	pub app_id: u32,
	pub mortality: Mortality,
	pub nonce: u64,
	pub tip: u128,
}

impl PopulatedOptions {
	pub async fn build(self) -> Params {
		let mut builder = AvailExtrinsicParamsBuilder::new();
		builder = builder.app_id(self.app_id);
		builder = builder.tip(self.tip);
		builder = builder.nonce(self.nonce);

		builder = builder.mortal_unchecked(
			self.mortality.block_height as u64,
			self.mortality.block_hash,
			self.mortality.period,
		);

		builder.build()
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
		Self {
			period,
			block_hash,
			block_height,
		}
	}

	pub async fn from_period(client: &Client, period: u64) -> Result<Self, RpcError> {
		let header = client.finalized_block_header().await?;
		let (block_hash, block_height) = (header.hash(), header.number());
		Ok(Self {
			period,
			block_hash,
			block_height,
		})
	}
}
