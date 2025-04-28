use super::{
	utils::{self, SubmissionStateError, SubmittedTransaction},
	Options, TransactionDetails,
};
use crate::{
	error::ClientError,
	from_substrate::{FeeDetails, RuntimeDispatchInfo},
	runtime_api, Client, WaitFor, H256,
};
use subxt::{
	blocks::StaticExtrinsic,
	ext::scale_encode::EncodeAsFields,
	tx::{DefaultPayload, Payload},
};
use subxt_signer::sr25519::Keypair;

#[derive(Debug, Clone)]
pub struct Transaction<T>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	client: Client,
	payload: DefaultPayload<T>,
}

impl<T> Transaction<T>
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

	pub async fn execute(&self, account: &Keypair, options: Options) -> Result<SubmittedTransaction, subxt::Error> {
		utils::sign_and_submit(&self.client, account, &self.payload, options).await
	}

	pub async fn execute_and_watch_inclusion(
		&self,
		account: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, SubmissionStateError> {
		utils::sign_submit_and_watch(&self.client, account, &self.payload, WaitFor::BlockInclusion, options).await
	}

	pub async fn execute_and_watch_finalization(
		&self,
		account: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, SubmissionStateError> {
		utils::sign_submit_and_watch(
			&self.client,
			account,
			&self.payload,
			WaitFor::BlockFinalization,
			options,
		)
		.await
	}
}
