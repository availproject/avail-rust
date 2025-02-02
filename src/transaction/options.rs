use crate::{
	error::ClientError,
	rpc::{chain::get_header, system::account_next_index},
	AccountId, AvailConfig, AvailExtrinsicParamsBuilder, Block, Client, H256,
};
use subxt::config::Header;

pub type Params =
	<<AvailConfig as subxt::Config>::ExtrinsicParams as subxt::config::ExtrinsicParams<
		AvailConfig,
	>>::Params;

#[derive(Debug, Clone, Copy)]
pub struct Options {
	pub app_id: Option<u32>,
	pub mortality: Option<Mortality>,
	pub nonce: Option<Nonce>,
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

	pub fn mortality(mut self, value: Mortality) -> Self {
		self.mortality = Some(value);
		self
	}

	pub fn nonce(mut self, value: Nonce) -> Self {
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
	) -> Result<PopulatedOptions, ClientError> {
		let app_id = self.app_id.unwrap_or_default();
		let tip = self.tip.unwrap_or_default();
		let nonce = self.nonce.unwrap_or(Nonce::BestBlock);
		let nonce = parse_nonce(client, nonce, account_id).await?;
		let mortality = self.mortality.unwrap_or(Mortality { period: 32 });
		let mortality = CheckedMortality::from_mortality(&mortality, client).await?;

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

#[derive(Debug, Clone, Copy)]
pub struct PopulatedOptions {
	pub app_id: u32,
	pub mortality: CheckedMortality,
	pub nonce: u64,
	pub tip: u128,
}

impl PopulatedOptions {
	pub async fn build(self) -> Result<Params, ClientError> {
		let mut builder = AvailExtrinsicParamsBuilder::new();
		builder = builder.app_id(self.app_id);
		builder = builder.tip(self.tip);
		builder = builder.nonce(self.nonce);

		builder = builder.mortal_unchecked(
			self.mortality.block_number as u64,
			self.mortality.block_hash,
			self.mortality.period,
		);

		Ok(builder.build())
	}

	pub async fn regenerate_mortality(&mut self, client: &Client) -> Result<(), ClientError> {
		let mortality = Mortality {
			period: self.mortality.period,
		};
		self.mortality = CheckedMortality::from_mortality(&mortality, client).await?;
		return Ok(());
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Mortality {
	pub period: u64,
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

	pub async fn from_mortality(m: &Mortality, client: &Client) -> Result<Self, ClientError> {
		let header = get_header(client, None).await?;
		let (block_hash, block_number) = (header.hash(), header.number());
		Ok(Self {
			period: m.period,
			block_hash,
			block_number,
		})
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Nonce {
	BestBlock,
	FinalizedBlock,
	Custom(u32),
}

pub async fn parse_nonce(
	client: &Client,
	nonce: Nonce,
	account_id: &AccountId,
) -> Result<u64, ClientError> {
	let nonce = match nonce {
		Nonce::FinalizedBlock => {
			let hash = Block::fetch_finalized_block_hash(client).await?;
			let block = client.blocks().at(hash).await?;
			block.account_nonce(account_id).await?
		},
		Nonce::BestBlock => account_next_index(client, account_id.to_string()).await? as u64,
		Nonce::Custom(x) => x as u64,
	};

	Ok(nonce)
}
