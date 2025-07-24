use super::Client;
use avail_rust_core::{
	DecodedTransaction, EncodeSelector, H256, HasTxDispatchIndex, HashNumber,
	rpc::{
		self,
		system::fetch_extrinsics_v1_types::{
			self as Types, ExtrinsicInformation, Filter, SignatureFilter, TransactionFilter,
		},
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

	/// Fetches specific transaction from a block.
	/// Transaction can be filtered by signature characteristics
	///
	/// By default, the api will request and return just the (Hex and SCALE decoded) Transaction Call
	/// If the full Transaction is needed or no data is needed then set `encode_as` to either
	/// `EncodeSelector::Extrinsic` or `EncodeSelector::None`
	pub async fn transaction(
		&self,
		block_id: HashNumber,
		transaction_id: HashNumber,
		encode_as: Option<EncodeSelector>,
	) -> Result<Option<Types::ExtrinsicInformation>, avail_rust_core::Error> {
		let transaction_filter = match transaction_id {
			HashNumber::Hash(item) => Types::TransactionFilter::TxHash(vec![item]),
			HashNumber::Number(item) => Types::TransactionFilter::TxIndex(vec![item]),
		};

		let mut result = self
			.transactions(block_id, Some(transaction_filter), None, encode_as)
			.await?;

		if result.is_empty() {
			return Ok(None);
		}

		Ok(Some(result.remove(0)))
	}

	/// Same as `transaction` but instead of returning on first error
	/// it tries it again a couple of time. After 6 failures is returns the original error.
	pub async fn transaction_with_retries(
		&self,
		block_id: HashNumber,
		transaction_id: HashNumber,
		encode_as: Option<EncodeSelector>,
	) -> Result<Option<Types::ExtrinsicInformation>, avail_rust_core::Error> {
		let transaction_filter = match transaction_id {
			HashNumber::Hash(item) => Types::TransactionFilter::TxHash(vec![item]),
			HashNumber::Number(item) => Types::TransactionFilter::TxIndex(vec![item]),
		};

		let mut result = self
			.transactions_with_retries(block_id, Some(transaction_filter), None, encode_as)
			.await?;

		if result.is_empty() {
			return Ok(None);
		}

		Ok(Some(result.remove(0)))
	}

	// Same as transaction but instead of returning encoded data + call information it returns
	// a fully decoded transaction.
	pub async fn transaction_static<T: HasTxDispatchIndex + codec::Decode>(
		&self,
		block_id: HashNumber,
		transaction_id: HashNumber,
	) -> Result<Option<(DecodedTransaction<T>, ExtrinsicInformation)>, avail_rust_core::Error> {
		let transaction_filter = match transaction_id {
			HashNumber::Hash(item) => Types::TransactionFilter::TxHash(vec![item]),
			HashNumber::Number(item) => Types::TransactionFilter::TxIndex(vec![item]),
		};
		let encode_as = Some(EncodeSelector::Extrinsic);

		let mut result = self
			.transactions(block_id, Some(transaction_filter), None, encode_as)
			.await?;

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

	/// Same as `transaction_static` but instead of returning on first error
	/// it tries it again a couple of time. After 6 failures is returns the original error.
	pub async fn transaction_static_with_retries<T: HasTxDispatchIndex + codec::Decode + From<Box<T>>>(
		&self,
		block_id: HashNumber,
		transaction_id: HashNumber,
	) -> Result<Option<DecodedTransaction<T>>, avail_rust_core::Error> {
		let transaction_filter = match transaction_id {
			HashNumber::Hash(item) => Types::TransactionFilter::TxHash(vec![item]),
			HashNumber::Number(item) => Types::TransactionFilter::TxIndex(vec![item]),
		};
		let encode_as = Some(EncodeSelector::Extrinsic);

		let mut result = self
			.transactions_with_retries(block_id, Some(transaction_filter), None, encode_as)
			.await?;

		if result.is_empty() {
			return Ok(None);
		}
		let ext = result.remove(0);
		let Some(encoded) = ext.encoded else {
			return Ok(None);
		};

		Ok(DecodedTransaction::try_from(encoded.as_str()).ok())
	}

	/// Fetches transactions from a block.
	/// Transactions can be filtered by transaction and/or signature characteristics
	///
	/// By default, the api will request and return just the (Hex and SCALE decoded) Transaction Call
	/// If the full Transaction is needed or no data is needed then set `encode_as` to either
	/// `EncodeSelector::Extrinsic` or `EncodeSelector::None`
	pub async fn transactions(
		&self,
		block_id: HashNumber,
		transaction_filter: Option<TransactionFilter>,
		signature_filter: Option<SignatureFilter>,
		encode_as: Option<EncodeSelector>,
	) -> Result<Types::Output, avail_rust_core::Error> {
		let filter = Filter::new(transaction_filter, signature_filter);
		let encode_as = encode_as.unwrap_or(EncodeSelector::Call);
		let options = Types::Options::new(Some(filter), Some(encode_as));

		self.client
			.rpc_api()
			.system_fetch_extrinsics_v1(block_id, Some(options))
			.await
	}

	/// Same as `transactions` but instead of returning on first error
	/// it tries it again a couple of time. After 6 failures is returns the original error.
	pub async fn transactions_with_retries(
		&self,
		block_id: HashNumber,
		transaction_filter: Option<TransactionFilter>,
		signature_filter: Option<SignatureFilter>,
		encode_as: Option<EncodeSelector>,
	) -> Result<Types::Output, avail_rust_core::Error> {
		let filter = Filter::new(transaction_filter, signature_filter);
		let encode_as = encode_as.unwrap_or(EncodeSelector::Call);
		let options = Types::Options::new(Some(filter), Some(encode_as));

		self.client
			.rpc_api()
			.system_fetch_extrinsics_v1_with_retries(block_id, Some(options))
			.await
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
	pub async fn rpc_block_with_retries(
		&self,
		at: H256,
	) -> Result<Option<rpc::BlockWithJustifications>, avail_rust_core::Error> {
		self.client.block_with_retries(at).await
	}
}
