use super::Client;
use avail_rust_core::{
	DecodedTransaction, EncodeSelector, H256, HasHeader, HashNumber,
	rpc::{
		BlockWithJustifications,
		system::fetch_extrinsics::{self, ExtrinsicInfo, TransactionFilter},
	},
};

#[derive(Clone)]
pub struct BlockClient {
	client: Client,
}

impl BlockClient {
	pub fn new(client: Client) -> Self {
		Self { client }
	}

	/// TODO
	pub async fn transaction(
		&self,
		block_id: HashNumber,
		transaction_id: HashNumber,
		encode_as: EncodeSelector,
	) -> Result<Option<ExtrinsicInfo>, avail_rust_core::Error> {
		let mut builder = self.builder().encode_as(encode_as).retry_on_error(true);

		builder = match transaction_id {
			HashNumber::Hash(item) => builder.tx_hash(item),
			HashNumber::Number(item) => builder.tx_index(item),
		};

		let result = builder.fetch(block_id).await?;
		Ok(result.first().cloned())
	}

	// Same as transaction but instead of returning encoded data + call information it returns
	// a fully decoded transaction.
	pub async fn transaction_static<T: HasHeader + codec::Decode>(
		&self,
		block_id: HashNumber,
		transaction_id: HashNumber,
	) -> Result<Option<(DecodedTransaction<T>, ExtrinsicInfo)>, avail_rust_core::Error> {
		let mut builder = self.builder().encode_as(EncodeSelector::Extrinsic).retry_on_error(true);

		builder = match transaction_id {
			HashNumber::Hash(item) => builder.tx_hash(item),
			HashNumber::Number(item) => builder.tx_index(item),
		};

		let mut result = builder.fetch(block_id).await?;
		if result.is_empty() {
			return Ok(None);
		}
		let mut info = result.remove(0);
		let Some(data) = info.data.take() else {
			return Err("Fetch extrinsics endpoint returned an extrinsic with no data.".into());
		};

		let Ok(decoded) = DecodedTransaction::<T>::try_from(data.as_str()) else {
			return Ok(None);
		};

		Ok(Some((decoded, info)))
	}

	/// TODO DOC
	pub fn builder(&self) -> BlockTransactionsBuilder {
		BlockTransactionsBuilder::new(self.client.clone())
	}

	/// Fetches a block at a specific block hash
	/// A block contains a block header, all the transactions and all the justifications
	///
	/// In 99.99% cases `transactions` or `transaction` method is the one that you need/want
	pub async fn rpc_block(&self, at: H256) -> Result<Option<BlockWithJustifications>, avail_rust_core::Error> {
		self.client.block(at).await
	}

	/// Same as `rpc_block` but instead of returning on first error
	/// it tries it again a couple of time. After 6 failures is returns the original error.
	pub async fn rpc_block_ext(
		&self,
		at: H256,
		retry_on_error: bool,
		retry_on_none: bool,
	) -> Result<Option<BlockWithJustifications>, avail_rust_core::Error> {
		self.client.block_ext(at, retry_on_error, retry_on_none).await
	}
}

#[derive(Clone)]
pub struct BlockTransactionsBuilder {
	client: Client,
	options: fetch_extrinsics::Options,
	retry_on_error: bool,
}

impl BlockTransactionsBuilder {
	pub fn new(client: Client) -> Self {
		Self { client, options: Default::default(), retry_on_error: false }
	}

	pub fn reset(mut self) -> Self {
		self.options = Default::default();
		self.retry_on_error = false;
		self
	}

	pub fn tx_filter(mut self, value: TransactionFilter) -> Self {
		self.options.transaction_filter = value;
		self
	}

	pub fn tx_hash(self, value: H256) -> Self {
		self.tx_hashes(vec![value])
	}

	pub fn tx_hashes(mut self, value: Vec<H256>) -> Self {
		self.options.transaction_filter = TransactionFilter::TxHash(value);
		self
	}

	pub fn tx_index(self, value: u32) -> Self {
		self.tx_indexes(vec![value])
	}

	pub fn tx_indexes(mut self, value: Vec<u32>) -> Self {
		self.options.transaction_filter = TransactionFilter::TxIndex(value);
		self
	}

	pub fn pallet(self, value: u8) -> Self {
		self.pallets(vec![value])
	}

	pub fn pallets(mut self, value: Vec<u8>) -> Self {
		self.options.transaction_filter = TransactionFilter::Pallet(value);
		self
	}

	pub fn call(self, value: (u8, u8)) -> Self {
		self.calls(vec![value])
	}

	pub fn calls(mut self, value: Vec<(u8, u8)>) -> Self {
		self.options.transaction_filter = TransactionFilter::PalletCall(value);
		self
	}

	pub fn ss58_address(mut self, value: Option<String>) -> Self {
		self.options.ss58_address = value;
		self
	}

	pub fn nonce(mut self, value: Option<u32>) -> Self {
		self.options.nonce = value;
		self
	}

	pub fn app_id(mut self, value: Option<u32>) -> Self {
		self.options.app_id = value;
		self
	}

	pub fn encode_as(mut self, value: EncodeSelector) -> Self {
		self.options.encode_as = value;
		self
	}

	pub fn retry_on_error(mut self, value: bool) -> Self {
		self.retry_on_error = value;
		self
	}

	pub async fn fetch(&self, block_id: HashNumber) -> Result<Vec<ExtrinsicInfo>, avail_rust_core::Error> {
		self.client
			.rpc_api()
			.system_fetch_extrinsics_v1_ext(block_id, self.options.clone(), self.retry_on_error)
			.await
	}
}
