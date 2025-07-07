use super::Client;
use avail_rust_core::{
	FetchExtrinsicsV1Options, H256, HashNumber,
	rpc::{self, system::fetch_extrinsics_v1_types as Types},
};

#[derive(Clone)]
pub struct BlockClient {
	client: Client,
}

impl BlockClient {
	pub fn new(client: Client) -> Self {
		Self { client }
	}

	pub async fn block_transaction(
		&self,
		block_id: HashNumber,
		transaction_id: HashNumber,
		sig_filter: Option<Types::SignatureFilter>,
		selector: Option<Types::EncodeSelector>,
	) -> Result<Option<Types::ExtrinsicInformation>, avail_rust_core::Error> {
		let filter = match transaction_id {
			HashNumber::Hash(item) => Types::TransactionFilter::TxHash(vec![item]),
			HashNumber::Number(item) => Types::TransactionFilter::TxIndex(vec![item]),
		};
		let filter = Some(Types::Filter::new(Some(filter), sig_filter));
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

	pub async fn block_transactions(
		&self,
		block_id: HashNumber,
		options: Option<Types::Options>,
	) -> Result<Types::Output, avail_rust_core::Error> {
		self.client
			.rpc_api()
			.system_fetch_extrinsics_v1(block_id, options)
			.await
	}

	pub async fn rpc_block(&self, at: H256) -> Result<Option<rpc::Block>, avail_rust_core::Error> {
		self.client.block(at).await.map(|b| b.map(|x| x.block))
	}

	pub async fn rpc_block_with_justifications(
		&self,
		at: H256,
	) -> Result<Option<rpc::BlockWithJustifications>, avail_rust_core::Error> {
		self.client.block(at).await
	}

	pub async fn rpc_block_justifications(
		&self,
		at: H256,
	) -> Result<Option<Vec<rpc::BlockJustification>>, avail_rust_core::Error> {
		self.client.block(at).await.map(|b| b.and_then(|x| x.justifications))
	}
}
