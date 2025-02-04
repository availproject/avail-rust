use super::{utils, Options, TransactionDetails};
use crate::{
	error::ClientError,
	from_substrate::{FeeDetails, RuntimeDispatchInfo},
	runtime_api, Client, WaitFor, H256,
};
use std::time::Duration;
use subxt::{
	blocks::StaticExtrinsic,
	ext::scale_encode::EncodeAsFields,
	tx::{DefaultPayload, Payload},
};
use subxt_signer::sr25519::Keypair;

pub trait WebSocket {
	#[allow(async_fn_in_trait)]
	async fn execute_and_watch_inclusion(
		&self,
		account: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, ClientError>;

	#[allow(async_fn_in_trait)]
	async fn execute_and_watch_finalization(
		&self,
		account: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, ClientError>;

	#[allow(async_fn_in_trait)]
	async fn execute_and_watch(
		&self,
		wait_for: WaitFor,
		account: &Keypair,
		options: Options,
		block_timeout: Option<u32>,
	) -> Result<TransactionDetails, ClientError>;

	#[allow(async_fn_in_trait)]
	async fn execute(&self, account: &Keypair, options: Options) -> Result<H256, ClientError>;
}

pub trait HTTP {
	#[allow(async_fn_in_trait)]
	async fn execute_and_watch_inclusion(
		&self,
		account: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, ClientError>;

	#[allow(async_fn_in_trait)]
	async fn execute_and_watch_finalization(
		&self,
		account: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, ClientError>;

	#[allow(async_fn_in_trait)]
	async fn execute_and_watch(
		&self,
		wait_for: WaitFor,
		account: &Keypair,
		options: Options,
		block_timeout: Option<u32>,
		sleep_duration: Option<Duration>,
	) -> Result<TransactionDetails, ClientError>;

	#[allow(async_fn_in_trait)]
	async fn execute(&self, account: &Keypair, options: Options) -> Result<H256, ClientError>;
}

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

		let params = options.build().await?;
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

		let params = options.build().await?;
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
}

impl<T> WebSocket for Transaction<T>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	async fn execute(&self, account: &Keypair, options: Options) -> Result<H256, ClientError> {
		utils::sign_and_send(&self.client, account, &self.payload, options).await
	}

	async fn execute_and_watch_inclusion(
		&self,
		account: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, ClientError> {
		WebSocket::execute_and_watch(self, WaitFor::BlockInclusion, account, options, Some(3)).await
	}

	async fn execute_and_watch_finalization(
		&self,
		account: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, ClientError> {
		WebSocket::execute_and_watch(self, WaitFor::BlockFinalization, account, options, Some(5)).await
	}

	async fn execute_and_watch(
		&self,
		wait_for: WaitFor,
		account: &Keypair,
		options: Options,
		block_timeout: Option<u32>,
	) -> Result<TransactionDetails, ClientError> {
		utils::sign_send_and_watch(
			&self.client,
			account,
			&self.payload,
			wait_for,
			options,
			block_timeout,
			Some(3),
		)
		.await
	}
}

impl<T> HTTP for Transaction<T>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	async fn execute(&self, account: &Keypair, options: Options) -> Result<H256, ClientError> {
		utils::http_sign_and_send(&self.client, account, &self.payload, options).await
	}

	async fn execute_and_watch_inclusion(
		&self,
		account: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, ClientError> {
		HTTP::execute_and_watch(self, WaitFor::BlockInclusion, account, options, Some(2), None).await
	}

	async fn execute_and_watch_finalization(
		&self,
		account: &Keypair,
		options: Options,
	) -> Result<TransactionDetails, ClientError> {
		HTTP::execute_and_watch(self, WaitFor::BlockFinalization, account, options, Some(5), None).await
	}

	async fn execute_and_watch(
		&self,
		wait_for: WaitFor,
		account: &Keypair,
		options: Options,
		block_timeout: Option<u32>,
		sleep_duration: Option<Duration>,
	) -> Result<TransactionDetails, ClientError> {
		utils::http_sign_send_and_watch(
			&self.client,
			account,
			&self.payload,
			wait_for,
			options,
			block_timeout,
			Some(3),
			sleep_duration,
		)
		.await
	}
}
