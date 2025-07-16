use super::Client;
use avail_rust_core::{
	EncodeSelector, FetchExtrinsicsV1Options, H256, HashNumber,
	rpc::{
		self,
		system::fetch_extrinsics_v1_types::{self as Types, Filter, SignatureFilter, TransactionFilter},
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
	pub async fn block_transaction(
		&self,
		block_id: HashNumber,
		transaction_id: HashNumber,
		signature_filter: Option<SignatureFilter>,
		selector: Option<EncodeSelector>,
	) -> Result<Option<Types::ExtrinsicInformation>, avail_rust_core::Error> {
		let filter = match transaction_id {
			HashNumber::Hash(item) => Types::TransactionFilter::TxHash(vec![item]),
			HashNumber::Number(item) => Types::TransactionFilter::TxIndex(vec![item]),
		};
		let filter = Some(Types::Filter::new(Some(filter), signature_filter));
		let params = FetchExtrinsicsV1Options::new(filter, selector);
		let mut result = self
			.client
			.rpc_api()
			.system_fetch_extrinsics_v1(block_id, Some(params))
			.await?;

		if result.is_empty() {
			return Ok(None);
		}

		Ok(Some(result.remove(0)))
	}

	/// Fetches transactions from a block.
	/// Transactions can be filtered by transaction and/or signature characteristics
	///
	/// By default, the api will request and return just the (Hex and SCALE decoded) Transaction Call
	/// If the full Transaction is needed or no data is needed then set `encode_as` to either
	/// `EncodeSelector::Extrinsic` or `EncodeSelector::None`
	pub async fn block_transactions(
		&self,
		block_id: HashNumber,
		transaction_filter: Option<TransactionFilter>,
		signature_filter: Option<SignatureFilter>,
		encode_as: Option<EncodeSelector>,
	) -> Result<Types::Output, avail_rust_core::Error> {
		let filter = Filter::new(transaction_filter, signature_filter);
		let options = Types::Options::new(Some(filter), encode_as);

		self.client
			.rpc_api()
			.system_fetch_extrinsics_v1(block_id, Some(options))
			.await
	}

	/// Fetches a block at a specific block hash
	/// A block contains a block header, all the transactions and all the justifications
	///
	/// In 99.99% cases `.block_transaction` or `.block_transactions` method is the one that you need/want
	pub async fn rpc_block(&self, at: H256) -> Result<Option<rpc::BlockWithJustifications>, avail_rust_core::Error> {
		self.client.block(at).await
	}
}
