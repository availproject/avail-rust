use crate::{rpc::system::account_next_index, AccountId, AvailConfig, AvailExtrinsicParamsBuilder, Client, H256};
use subxt::config::Header;

pub type Params =
	<<AvailConfig as subxt::Config>::ExtrinsicParams as subxt::config::ExtrinsicParams<AvailConfig>>::Params;

#[derive(Debug, Clone, Copy)]
pub struct Options {
	pub app_id: Option<u32>,
	pub mortality: Option<u64>,
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

	pub fn mortality(mut self, value: u64) -> Self {
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

	pub async fn build(self, client: &Client, account_id: &AccountId) -> Result<PopulatedOptions, subxt::Error> {
		let app_id = self.app_id.unwrap_or_default();
		let tip = self.tip.unwrap_or_default();
		let nonce = parse_nonce(client, self.nonce, account_id).await?;
		let period = self.mortality.unwrap_or(32);
		let mortality = CheckedMortality::from_period(period, client).await?;

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
	pub mortality: CheckedMortality,
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
			self.mortality.block_number as u64,
			self.mortality.block_hash,
			self.mortality.period,
		);

		builder.build()
	}
}

#[derive(Debug, Clone, Copy)]
pub struct CheckedMortality {
	pub period: u64,
	pub block_hash: H256,
	pub block_number: u32,
}
impl CheckedMortality {
	pub fn new(period: u64, block_hash: H256, block_number: u32) -> Self {
		Self {
			period,
			block_hash,
			block_number,
		}
	}

	pub async fn from_period(period: u64, client: &Client) -> Result<Self, subxt::Error> {
		let finalized_hash = client.finalized_block_hash().await?;
		let header = client.header_at(finalized_hash).await?;
		let (block_hash, block_number) = (header.hash(), header.number());
		Ok(Self {
			period,
			block_hash,
			block_number,
		})
	}
}

pub async fn parse_nonce(client: &Client, nonce: Option<u32>, account_id: &AccountId) -> Result<u64, subxt::Error> {
	let nonce = match nonce {
		Some(x) => x as u64,
		None => account_next_index(client, account_id.to_string()).await? as u64,
	};

	Ok(nonce)
}
