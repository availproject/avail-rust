use super::{
	submitting::{SubmissionStateError, SubmittedTransaction},
	Options,
};
use crate::{
	block::EventRecords,
	block_transaction::Filter,
	error::ClientError,
	from_substrate::{FeeDetails, RuntimeDispatchInfo},
	runtime_api, Client, H256,
};
use std::sync::Arc;
use subxt::{
	blocks::StaticExtrinsic,
	ext::scale_encode::EncodeAsFields,
	tx::{DefaultPayload, Payload},
};
use subxt_signer::sr25519::Keypair;

#[derive(Debug, Clone)]
pub struct SubmittableTransaction<T>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	client: Client,
	payload: DefaultPayload<T>,
}

impl<T> SubmittableTransaction<T>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	pub fn new(client: Client, payload: DefaultPayload<T>) -> Self {
		Self { client, payload }
	}

	pub async fn payment_query_info(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<RuntimeDispatchInfo, ClientError> {
		let account_id = account.public_key().to_account_id();
		let options = options.unwrap_or_default().build(&self.client, &account_id).await?;

		let params = options.build().await;
		let tx = self
			.client
			.online_client
			.tx()
			.create_signed(&self.payload, account, params)
			.await?;

		let tx = tx.encoded();

		runtime_api::transaction_payment::query_info(&self.client, tx.to_vec(), None).await
	}

	pub async fn payment_query_fee_details(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<FeeDetails, ClientError> {
		let account_id = account.public_key().to_account_id();
		let options = options.unwrap_or_default().build(&self.client, &account_id).await?;

		let params = options.build().await;
		let tx = self
			.client
			.online_client
			.tx()
			.create_signed(&self.payload, account, params)
			.await?;

		let tx = tx.encoded();

		runtime_api::transaction_payment::query_fee_details(&self.client, tx.to_vec(), None).await
	}

	pub async fn payment_query_call_info(&self) -> Result<RuntimeDispatchInfo, ClientError> {
		let metadata = self.client.online_client.metadata();
		let call = self.payload.encode_call_data(&metadata)?;

		runtime_api::transaction_payment::query_call_info(&self.client, call, None).await
	}

	pub async fn payment_query_call_fee_details(&self) -> Result<FeeDetails, ClientError> {
		let metadata = self.client.online_client.metadata();
		let call = self.payload.encode_call_data(&metadata)?;

		runtime_api::transaction_payment::query_call_fee_details(&self.client, call, None).await
	}

	pub async fn execute(&self, signer: &Keypair, options: Options) -> Result<SubmittedTransaction, subxt::Error> {
		super::submitting::sign_and_submit(&self.client, signer, &self.payload, options).await
	}

	pub async fn execute_and_watch(
		&self,
		signer: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, SubmissionStateError> {
		super::submitting::sign_submit_and_watch(&self.client, signer, &self.payload, options).await
	}
}

#[derive(Debug, Clone)]
pub struct TransactionDetails {
	client: Client,
	pub events: Option<Arc<EventRecords>>,
	pub tx_hash: H256,
	pub tx_index: u32,
	pub block_hash: H256,
	pub block_number: u32,
}

impl TransactionDetails {
	pub fn new(
		client: Client,
		events: Option<EventRecords>,
		tx_hash: H256,
		tx_index: u32,
		block_hash: H256,
		block_number: u32,
	) -> Self {
		let events = events.map(|x| x.into());
		Self {
			client,
			events,
			tx_hash,
			tx_index,
			block_hash,
			block_number,
		}
	}

	/// Returns None if it was not possible to determine if the transaction was successful or not
	/// If Some is returned then
	///    true means the transaction was successful
	///    false means the transaction failed
	pub fn is_successful(&self) -> Option<bool> {
		match &self.events {
			Some(events) => Some(events.has_system_extrinsic_success()),
			None => None,
		}
	}

	/// Returns Err if it was not possible to determine if the transaction was decodable
	/// If Ok is returned then
	///    Some means the transaction was successfully decoded
	///    None means the transaction cannot be decoded as T
	pub async fn decode_as<T: StaticExtrinsic + Clone>(&self) -> Result<Option<T>, ClientError> {
		let block = crate::block::Block::new(&self.client, self.block_hash).await?;
		let filter = Filter::new().tx_index(self.tx_index);
		let txs = block.transactions_static::<T>(filter);
		if txs.is_empty() {
			return Ok(None);
		}
		Ok(Some(txs[0].value.clone()))
	}

	/// Returns Err if it was not possible to determine if the transaction was decodable
	/// If Ok is returned then
	///    true means the transaction was successfully decoded
	///    false means the transaction cannot be decoded as T
	pub async fn is<T: StaticExtrinsic + Clone>(&self) -> Result<bool, ClientError> {
		let block = crate::block::Block::new(&self.client, self.block_hash).await?;
		let filter = Filter::new().tx_index(self.tx_index);
		let txs = block.transactions_static::<T>(filter);
		Ok(!txs.is_empty())
	}
}
