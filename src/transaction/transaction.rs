use super::{utils, Options, TransactionDetails};
use crate::{
	error::ClientError, from_substrate::FeeDetails, rpc::payment::query_fee_details, AOnlineClient,
	WaitFor, H256,
};
use subxt::{
	backend::rpc::RpcClient, blocks::StaticExtrinsic, ext::scale_encode::EncodeAsFields,
	tx::DefaultPayload,
};
use subxt_signer::sr25519::Keypair;

#[derive(Debug, Clone)]
pub struct Transaction<T>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	online_client: AOnlineClient,
	rpc_client: RpcClient,
	payload: DefaultPayload<T>,
}

impl<T> Transaction<T>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	pub fn new(
		online_client: AOnlineClient,
		rpc_client: RpcClient,
		payload: DefaultPayload<T>,
	) -> Self {
		Self {
			online_client,
			rpc_client,
			payload,
		}
	}

	pub async fn execute_and_watch_inclusion(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<TransactionDetails, ClientError> {
		self.execute_and_watch(WaitFor::BlockInclusion, account, options, Some(2))
			.await
	}

	pub async fn execute_and_watch_finalization(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<TransactionDetails, ClientError> {
		self.execute_and_watch(WaitFor::BlockFinalization, account, options, Some(5))
			.await
	}

	pub async fn execute_and_watch(
		&self,
		wait_for: WaitFor,
		account: &Keypair,
		options: Option<Options>,
		block_timeout: Option<u32>,
	) -> Result<TransactionDetails, ClientError> {
		utils::sign_send_and_watch(
			&self.online_client,
			&self.rpc_client,
			account,
			&self.payload,
			wait_for,
			options,
			block_timeout,
			Some(3),
		)
		.await
	}

	pub async fn execute(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<H256, ClientError> {
		utils::sign_and_send(
			&self.online_client,
			&self.rpc_client,
			account,
			&self.payload,
			options,
		)
		.await
	}

	pub async fn payment_query_info(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<u128, ClientError> {
		let account_id = account.public_key().to_account_id();
		let options = options
			.unwrap_or_default()
			.build(&self.online_client, &self.rpc_client, &account_id)
			.await?;

		let params = options.build(&self.rpc_client).await?;
		let tx = self
			.online_client
			.tx()
			.create_signed(&self.payload, account, params)
			.await?;

		Ok(tx.partial_fee_estimate().await?)
	}

	pub async fn payment_query_fee_details(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<FeeDetails, ClientError> {
		let account_id = account.public_key().to_account_id();
		let options = options
			.unwrap_or_default()
			.build(&self.online_client, &self.rpc_client, &account_id)
			.await?;

		let params = options.build(&self.rpc_client).await?;
		let tx = self
			.online_client
			.tx()
			.create_signed(&self.payload, account, params)
			.await?;

		let len_bytes: [u8; 4] = (tx.encoded().len() as u32).to_le_bytes();
		let encoded_with_len = [tx.encoded(), &len_bytes[..]].concat();

		query_fee_details(&self.rpc_client, encoded_with_len.into(), None).await
	}
}

impl<T> Transaction<T>
where
	T: StaticExtrinsic + EncodeAsFields,
{
	pub async fn http_execute_and_watch_inclusion(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<TransactionDetails, ClientError> {
		self.http_execute_and_watch(WaitFor::BlockInclusion, account, options, Some(2))
			.await
	}

	pub async fn http_execute_and_watch_finalization(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<TransactionDetails, ClientError> {
		self.http_execute_and_watch(WaitFor::BlockFinalization, account, options, Some(5))
			.await
	}

	pub async fn http_execute_and_watch(
		&self,
		wait_for: WaitFor,
		account: &Keypair,
		options: Option<Options>,
		block_timeout: Option<u32>,
	) -> Result<TransactionDetails, ClientError> {
		utils::http_sign_send_and_watch(
			&self.online_client,
			&self.rpc_client,
			account,
			&self.payload,
			wait_for,
			options,
			block_timeout,
			Some(3),
		)
		.await
	}

	pub async fn http_execute(
		&self,
		account: &Keypair,
		options: Option<Options>,
	) -> Result<H256, ClientError> {
		utils::http_sign_and_send(
			&self.online_client,
			&self.rpc_client,
			account,
			&self.payload,
			options,
		)
		.await
	}
}
