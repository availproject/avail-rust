use super::Client;
use avail_rust_core::{
	DecodedTransaction, EncodeSelector, H256, HasTxDispatchIndex, HashNumber, rpc,
	rpc::system::fetch_extrinsics_v1_types::{self as Types, ExtrinsicInformation, SignatureFilter, TransactionFilter},
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
		encode_as: Option<EncodeSelector>,
	) -> Result<Option<Types::ExtrinsicInformation>, avail_rust_core::Error> {
		let mut builder = self
			.builder()
			.encode_as(encode_as.unwrap_or_default())
			.retry_on_error(true);

		builder = match transaction_id {
			HashNumber::Hash(item) => builder.tx_hash(item),
			HashNumber::Number(item) => builder.tx_index(item),
		};

		let result = builder.fetch(block_id).await?;
		Ok(result.first().cloned())
	}

	// Same as transaction but instead of returning encoded data + call information it returns
	// a fully decoded transaction.
	pub async fn transaction_static<T: HasTxDispatchIndex + codec::Decode>(
		&self,
		block_id: HashNumber,
		transaction_id: HashNumber,
	) -> Result<Option<(DecodedTransaction<T>, ExtrinsicInformation)>, avail_rust_core::Error> {
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
		let Some(encoded) = info.encoded.take() else {
			return Ok(None);
		};

		let Ok(decoded) = DecodedTransaction::<T>::try_from(encoded.as_str()) else {
			return Ok(None);
		};

		Ok(Some((decoded, info)))
	}

	/// TODO
	pub fn builder(&self) -> BlockTransactionsBuilder {
		BlockTransactionsBuilder::new(self.client.clone())
	}

	/// Fetches a block at a specific block hash
	/// A block contains a block header, all the transactions and all the justifications
	///
	/// In 99.99% cases `transactions` or `transaction` method is the one that you need/want
	pub async fn rpc_block(&self, at: H256) -> Result<Option<rpc::BlockWithJustifications>, avail_rust_core::Error> {
		self.client.block(at).await
	}

	/// Same as `rpc_block` but instead of returning on first error
	/// it tries it again a couple of time. After 6 failures is returns the original error.
	pub async fn rpc_block_ext(
		&self,
		at: H256,
		retry_on_error: bool,
		retry_on_none: bool,
	) -> Result<Option<rpc::BlockWithJustifications>, avail_rust_core::Error> {
		self.client.block_ext(at, retry_on_error, retry_on_none).await
	}
}

#[derive(Clone)]
pub struct BlockTransactionsBuilder {
	client: Client,
	transaction_filter: TransactionFilter,
	signature_filter: SignatureFilter,
	encode_as: EncodeSelector,
	retry_on_error: bool,
}

impl BlockTransactionsBuilder {
	pub fn new(client: Client) -> Self {
		Self {
			client,
			transaction_filter: Default::default(),
			signature_filter: Default::default(),
			encode_as: Default::default(),
			retry_on_error: false,
		}
	}

	pub fn reset(mut self) -> Self {
		self.transaction_filter = Default::default();
		self.signature_filter = Default::default();
		self.encode_as = Default::default();
		self.retry_on_error = false;
		self
	}

	pub fn transaction_filter(mut self, value: TransactionFilter) -> Self {
		self.transaction_filter = value;
		self
	}

	pub fn signature_filter(mut self, value: SignatureFilter) -> Self {
		self.signature_filter = value;
		self
	}

	pub fn tx_hash(self, value: H256) -> Self {
		self.tx_hashes(vec![value])
	}

	pub fn tx_hashes(mut self, value: Vec<H256>) -> Self {
		self.transaction_filter = TransactionFilter::TxHash(value);
		self
	}

	pub fn tx_index(self, value: u32) -> Self {
		self.tx_indexes(vec![value])
	}

	pub fn tx_indexes(mut self, value: Vec<u32>) -> Self {
		self.transaction_filter = TransactionFilter::TxIndex(value);
		self
	}

	pub fn pallet(self, value: u8) -> Self {
		self.pallets(vec![value])
	}

	pub fn pallets(mut self, value: Vec<u8>) -> Self {
		self.transaction_filter = TransactionFilter::Pallet(value);
		self
	}

	pub fn call(self, value: (u8, u8)) -> Self {
		self.calls(vec![value])
	}

	pub fn calls(mut self, value: Vec<(u8, u8)>) -> Self {
		self.transaction_filter = TransactionFilter::PalletCall(value);
		self
	}

	pub fn ss58_address(mut self, value: Option<String>) -> Self {
		self.signature_filter.ss58_address = value;
		self
	}

	pub fn none(mut self, value: Option<u32>) -> Self {
		self.signature_filter.nonce = value;
		self
	}

	pub fn app_id(mut self, value: Option<u32>) -> Self {
		self.signature_filter.app_id = value;
		self
	}

	pub fn encode_as(mut self, value: EncodeSelector) -> Self {
		self.encode_as = value;
		self
	}

	pub fn retry_on_error(mut self, value: bool) -> Self {
		self.retry_on_error = value;
		self
	}

	pub async fn fetch(&self, block_id: HashNumber) -> Result<Types::Output, avail_rust_core::Error> {
		let filter = Types::Filter {
			signature: Some(self.signature_filter.clone()),
			transaction: Some(self.transaction_filter.clone()),
		};
		let options = Types::Options { filter: Some(filter), encode_selector: Some(self.encode_as) };

		self.client
			.rpc_api()
			.system_fetch_extrinsics_v1_ext(block_id, options.clone(), self.retry_on_error)
			.await
	}
}
