use super::{block_client::BlockClient, event_client::EventClient, online_client::OnlineClientT};
use crate::{
	BlockState, avail,
	clients::{
		runtime_api::RuntimeApi,
		utils::{with_retry_on_error, with_retry_on_error_and_none},
	},
	subscription::{self, Subscription},
	subxt_rpcs::RpcClient,
	subxt_signer::sr25519::Keypair,
	transaction::SubmittedTransaction,
	transaction_options::Options,
	transactions::Transactions,
};
use avail::{
	balances::types::AccountData,
	system::{storage as SystemStorage, types::AccountInfo},
};
use avail_rust_core::{
	AccountId, AvailHeader, BlockRef, H256, HashNumber, StorageMap,
	grandpa::GrandpaJustification,
	rpc::{self, BlockWithJustifications, system},
};
use codec::Decode;
use std::time::Duration;

#[cfg(feature = "subxt")]
use crate::config::{ABlocksClient, AConstantsClient, AStorageClient};

#[derive(Clone)]
pub struct Client {
	#[cfg(not(feature = "subxt"))]
	online_client: super::online_client::OnlineClient,
	#[cfg(feature = "subxt")]
	online_client: crate::config::AOnlineClient,
	pub rpc_client: RpcClient,
}

impl Client {
	#[cfg(feature = "reqwest")]
	pub async fn new(endpoint: &str) -> Result<Client, avail_rust_core::Error> {
		let rpc_client = super::reqwest_client::ReqwestClient::new(endpoint);
		let rpc_client = RpcClient::new(rpc_client);

		Self::new_rpc_client(rpc_client).await
	}

	pub async fn new_rpc_client(rpc_client: RpcClient) -> Result<Client, avail_rust_core::Error> {
		#[cfg(not(feature = "subxt"))]
		let online_client = super::online_client::SimpleOnlineClient::new(&rpc_client).await?;
		#[cfg(feature = "subxt")]
		let online_client = crate::config::AOnlineClient::from_rpc_client(rpc_client.clone()).await?;

		Self::new_custom(rpc_client, online_client.into()).await
	}

	#[cfg(not(feature = "subxt"))]
	pub async fn new_custom(
		rpc_client: RpcClient,
		online_client: super::online_client::OnlineClient,
	) -> Result<Client, avail_rust_core::Error> {
		Ok(Self { online_client, rpc_client })
	}

	#[cfg(feature = "subxt")]
	pub async fn new_custom(
		rpc_client: RpcClient,
		online_client: crate::config::AOnlineClient,
	) -> Result<Client, avail_rust_core::Error> {
		Ok(Self { online_client, rpc_client })
	}

	pub fn tx(&self) -> Transactions {
		Transactions(self.clone())
	}

	#[cfg(feature = "tracing")]
	pub fn enable_tracing(enable_json_format: bool) {
		use tracing_subscriber::util::SubscriberInitExt;

		let builder = tracing_subscriber::fmt::SubscriberBuilder::default();
		if enable_json_format {
			let builder = builder.json();
			builder.finish().init();
		} else {
			builder.finish().init();
		}
	}

	// Mini Clients
	pub fn event_client(&self) -> EventClient {
		EventClient::new(self.clone())
	}

	pub fn block_client(&self) -> BlockClient {
		BlockClient::new(self.clone())
	}

	#[cfg(not(feature = "subxt"))]
	pub fn online_client(&self) -> super::online_client::OnlineClient {
		self.online_client.clone()
	}

	#[cfg(feature = "subxt")]
	pub fn online_client(&self) -> crate::config::AOnlineClient {
		self.online_client.clone()
	}

	// Subxt
	#[cfg(feature = "subxt")]
	pub fn subxt_blocks_client(&self) -> ABlocksClient {
		self.online_client.blocks()
	}

	#[cfg(feature = "subxt")]
	pub fn subxt_storage_client(&self) -> AStorageClient {
		self.online_client.storage()
	}

	#[cfg(feature = "subxt")]
	pub fn subxt_constants_client(&self) -> AConstantsClient {
		self.online_client.constants()
	}

	pub fn rpc(&self) -> Rpc {
		Rpc::new(self.clone())
	}

	pub fn best(&self) -> Best {
		Best::new(self.clone())
	}

	pub fn finalized(&self) -> Finalized {
		Finalized::new(self.clone())
	}

	// Api
	pub fn runtime_api(&self) -> RuntimeApi {
		RuntimeApi::new(self.clone())
	}

	// Subscription
	pub fn subscription_block_header(&self, sub: Subscription) -> subscription::HeaderSubscription {
		subscription::HeaderSubscription::new(self.clone(), sub)
	}

	pub fn subscription_block(&self, sub: Subscription) -> subscription::BlockSubscription {
		subscription::BlockSubscription::new(self.clone(), sub)
	}

	pub fn subscription_grandpa_justification(
		&self,
		block_height: u32,
		poll_rate: Duration,
	) -> subscription::GrandpaJustificationSubscription {
		subscription::GrandpaJustificationSubscription::new(self.clone(), poll_rate, block_height)
	}

	pub fn subscription_grandpa_json_justification(
		&self,
		block_height: u32,
		poll_rate: Duration,
	) -> subscription::GrandpaJustificationJsonSubscription {
		subscription::GrandpaJustificationJsonSubscription::new(self.clone(), poll_rate, block_height)
	}
}

pub struct Rpc {
	client: Client,
	retry_on_error: Option<bool>,
	retry_on_none: Option<bool>,
}
impl Rpc {
	pub fn new(client: Client) -> Self {
		Self { client, retry_on_error: None, retry_on_none: None }
	}

	pub fn retry_on(mut self, error: Option<bool>, none: Option<bool>) -> Self {
		self.retry_on_error = error;
		self.retry_on_none = none;
		self
	}

	pub async fn block_hash(&self, block_height: Option<u32>) -> Result<Option<H256>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::chain::get_block_hash(&self.client.rpc_client, block_height).await };
		with_retry_on_error_and_none(f, retry_on_error, retry_on_none, "")
			.await
			.map_err(|e| e.into())
	}

	pub async fn block_header(&self, at: Option<H256>) -> Result<Option<AvailHeader>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::chain::get_header(&self.client.rpc_client, at).await };
		with_retry_on_error_and_none(f, retry_on_error, retry_on_none, "")
			.await
			.map_err(|e| e.into())
	}

	pub async fn block(&self, at: Option<H256>) -> Result<Option<BlockWithJustifications>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::chain::get_block(&self.client.rpc_client, at).await };
		with_retry_on_error_and_none(f, retry_on_error, retry_on_none, "")
			.await
			.map_err(|e| e.into())
	}

	// Nonce
	pub async fn nonce(&self, account_id: &AccountId) -> Result<u32, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move { system::account_next_index(&self.client.rpc_client, &std::format!("{}", account_id)).await };
		with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
	}

	pub async fn block_nonce(&self, account_id: &AccountId, block_hash: H256) -> Result<u32, avail_rust_core::Error> {
		self.account_info(account_id, block_hash).await.map(|x| x.nonce)
	}

	// Balance
	pub async fn balance(&self, account_id: &AccountId, at: H256) -> Result<AccountData, avail_rust_core::Error> {
		self.account_info(account_id, at).await.map(|x| x.data)
	}

	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: &AccountId, at: H256) -> Result<AccountInfo, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move {
			SystemStorage::Account::fetch(&self.client.rpc_client, account_id, Some(at))
				.await
				.map(|x| x.unwrap_or_default())
		};
		with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
	}

	// Block State
	pub async fn block_state(&self, block_ref: BlockRef) -> Result<BlockState, avail_rust_core::Error> {
		let real_block_hash = self.block_hash(Some(block_ref.height)).await?;
		let Some(real_block_hash) = real_block_hash else {
			return Ok(BlockState::DoesNotExist);
		};

		let finalized_block_height = self.client.finalized().block_height().await?;
		if block_ref.height > finalized_block_height {
			return Ok(BlockState::Included);
		}

		if block_ref.hash != real_block_hash {
			return Ok(BlockState::Discarded);
		}

		Ok(BlockState::Finalized)
	}

	// Block Height
	pub async fn block_height(&self, at: H256) -> Result<Option<u32>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::system::get_block_number(&self.client.rpc_client, at).await };
		with_retry_on_error_and_none(f, retry_on_error, retry_on_none, "")
			.await
			.map_err(|e| e.into())
	}

	pub async fn block_info(&self, use_best_block: bool) -> Result<BlockRef, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move { rpc::system::latest_block_info(&self.client.rpc_client, use_best_block).await };
		with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
	}

	// Sign and submit
	pub async fn submit(&self, tx: &avail_rust_core::Transaction<'_>) -> Result<H256, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let encoded = tx.encode();
		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signed {
			if let avail_rust_core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "tx", "Submitting Transaction. Address: {}, Nonce: {}, App Id: {}", account_id, signed.tx_extra.nonce, signed.tx_extra.app_id);
			}
		}

		let enc_slice = encoded.as_slice();
		let f = || async move { rpc::author::submit_extrinsic(&self.client.rpc_client, enc_slice).await };
		let tx_hash = with_retry_on_error(f, retry_on_error, "").await?;

		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signed {
			if let avail_rust_core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "tx", "Transaction Submitted.  Address: {}, Nonce: {}, App Id: {}, Tx Hash: {:?},", account_id, signed.tx_extra.nonce, signed.tx_extra.app_id, tx_hash);
			}
		}

		Ok(tx_hash)
	}

	pub async fn sign_payload<'a>(
		&self,
		signer: &Keypair,
		tx_payload: avail_rust_core::TransactionPayload<'a>,
	) -> avail_rust_core::Transaction<'a> {
		use avail_rust_core::Transaction;

		let account_id = signer.public_key().to_account_id();
		let signature = tx_payload.sign(signer);
		let tx = Transaction::new(account_id, signature, tx_payload);

		tx
	}

	pub async fn sign_call<'a>(
		&self,
		signer: &Keypair,
		tx_call: &'a avail_rust_core::TransactionCall,
		options: Options,
	) -> Result<avail_rust_core::Transaction<'a>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(&self.client, &account_id, retry_on_error).await?;

		let tx_extra = avail_rust_core::TransactionExtra::from(&refined_options);
		let tx_additional = avail_rust_core::TransactionAdditional {
			spec_version: self.client.online_client.spec_version(),
			tx_version: self.client.online_client.transaction_version(),
			genesis_hash: self.client.online_client.genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = avail_rust_core::TransactionPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
		Ok(self.sign_payload(signer, tx_payload).await)
	}

	pub async fn sign_and_submit_payload(
		&self,
		signer: &Keypair,
		tx_payload: avail_rust_core::TransactionPayload<'_>,
	) -> Result<H256, avail_rust_core::Error> {
		use avail_rust_core::Transaction;

		let account_id = signer.public_key().to_account_id();
		let signature = tx_payload.sign(signer);
		let tx = Transaction::new(account_id, signature, tx_payload);
		let tx_hash = self.submit(&tx).await?;

		Ok(tx_hash)
	}

	pub async fn sign_and_submit_call(
		&self,
		signer: &Keypair,
		tx_call: &avail_rust_core::TransactionCall,
		options: Options,
	) -> Result<SubmittedTransaction, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(&self.client, &account_id, retry_on_error).await?;

		let tx_extra = avail_rust_core::TransactionExtra::from(&refined_options);
		let tx_additional = avail_rust_core::TransactionAdditional {
			spec_version: self.client.online_client.spec_version(),
			tx_version: self.client.online_client.transaction_version(),
			genesis_hash: self.client.online_client.genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = avail_rust_core::TransactionPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
		let tx_hash = self.sign_and_submit_payload(signer, tx_payload).await?;

		let value = SubmittedTransaction::new(self.client.clone(), tx_hash, account_id, refined_options, tx_additional);
		Ok(value)
	}

	// Rest
	pub async fn state_call(
		&self,
		method: &str,
		data: &[u8],
		at: Option<H256>,
	) -> Result<String, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move { rpc::state::call(&self.client.rpc_client, method, data, at).await };
		with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
	}

	pub async fn state_get_metadata(&self, at: Option<H256>) -> Result<Vec<u8>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move { rpc::state::get_metadata(&self.client.rpc_client, at).await };
		with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
	}

	pub async fn state_get_storage(
		&self,
		key: &str,
		at: Option<H256>,
	) -> Result<Option<Vec<u8>>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move { rpc::state::get_storage(&self.client.rpc_client, key, at).await };
		with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
	}

	pub async fn state_get_keys_paged(
		&self,
		prefix: Option<&str>,
		count: u32,
		start_key: Option<&str>,
		at: Option<H256>,
	) -> Result<Vec<String>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f =
			|| async move { rpc::state::get_keys_paged(&self.client.rpc_client, prefix, count, start_key, at).await };
		with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
	}

	pub async fn system_fetch_extrinsics(
		&self,
		block_id: HashNumber,
		options: system::fetch_extrinsics::Options,
	) -> Result<Vec<system::fetch_extrinsics::ExtrinsicInfo>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let opt = &options;

		let f = || async move { system::fetch_extrinsics_v1(&self.client.rpc_client, block_id, opt).await };
		with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
	}

	pub async fn system_fetch_events(
		&self,
		at: H256,
		options: system::fetch_events::Options,
	) -> Result<Vec<system::fetch_events::PhaseEvents>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let opt = &options;

		let f = || async move { system::fetch_events_v1(&self.client.rpc_client, at, opt).await };
		with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
	}

	pub async fn grandpa_block_justification(
		&self,
		at: u32,
	) -> Result<Option<GrandpaJustification>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let f = || async move { rpc::grandpa::block_justification(&self.client.rpc_client, at).await };
		let result = with_retry_on_error(f, retry_on_error, "").await?;

		let Some(result) = result else {
			return Ok(None);
		};

		let justification = const_hex::decode(result.trim_start_matches("0x"))
			.map_err(|x| avail_rust_core::Error::from(x.to_string()))?;

		let justification = GrandpaJustification::decode(&mut justification.as_slice()).map_err(|e| e.to_string())?;
		Ok(Some(justification))
	}

	pub async fn grandpa_block_justification_json(
		&self,
		at: u32,
	) -> Result<Option<GrandpaJustification>, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let f = || async move { rpc::grandpa::block_justification_json(&self.client.rpc_client, at).await };
		let result = with_retry_on_error(f, retry_on_error, "").await?;

		let Some(result) = result else {
			return Ok(None);
		};

		let justification: GrandpaJustification = serde_json::from_str(result.as_str()).map_err(|e| e.to_string())?;
		Ok(Some(justification))
	}
}

pub struct Best {
	client: Client,
	retry_on_error: Option<bool>,
	retry_on_none: Option<bool>,
}
impl Best {
	pub fn new(client: Client) -> Self {
		Self { client, retry_on_error: None, retry_on_none: None }
	}

	pub fn retry_on(mut self, error: Option<bool>, none: Option<bool>) -> Self {
		self.retry_on_error = error;
		self.retry_on_none = none;
		self
	}

	pub async fn block_header(&self) -> Result<AvailHeader, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(true);

		let block_hash = self.block_hash().await?;
		let block_header = self
			.client
			.rpc()
			.retry_on(Some(retry_on_error), Some(retry_on_none))
			.block_header(Some(block_hash))
			.await?;
		let Some(block_header) = block_header else {
			return Err("Failed to fetch best block header".into());
		};

		Ok(block_header)
	}

	pub async fn block(&self) -> Result<BlockWithJustifications, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(true);

		let block_hash = self.block_hash().await?;
		let block = self
			.client
			.rpc()
			.retry_on(Some(retry_on_error), Some(retry_on_none))
			.block(Some(block_hash))
			.await?;
		let Some(block) = block else {
			return Err("Best block not found".into());
		};

		Ok(block)
	}

	pub async fn block_nonce(&self, account_id: &AccountId) -> Result<u32, avail_rust_core::Error> {
		self.block_account_info(account_id).await.map(|v| v.nonce)
	}

	pub async fn block_balance(&self, account_id: &AccountId) -> Result<AccountData, avail_rust_core::Error> {
		self.block_account_info(account_id).await.map(|x| x.data)
	}

	pub async fn block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let at = self.block_hash().await?;
		self.client
			.rpc()
			.retry_on(Some(retry_on_error), None)
			.account_info(account_id, at)
			.await
	}

	pub async fn block_info(&self) -> Result<BlockRef, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		self.client
			.rpc()
			.retry_on(Some(retry_on_error), None)
			.block_info(true)
			.await
	}

	pub async fn block_hash(&self) -> Result<H256, avail_rust_core::Error> {
		self.block_info().await.map(|x| x.hash)
	}

	pub async fn block_height(&self) -> Result<u32, avail_rust_core::Error> {
		self.block_info().await.map(|x| x.height)
	}
}

pub struct Finalized {
	client: Client,
	retry_on_error: Option<bool>,
	retry_on_none: Option<bool>,
}

impl Finalized {
	pub fn new(client: Client) -> Self {
		Self { client, retry_on_error: None, retry_on_none: None }
	}

	pub fn retry_on(mut self, error: Option<bool>, none: Option<bool>) -> Self {
		self.retry_on_error = error;
		self.retry_on_none = none;
		self
	}

	pub async fn block_header(&self) -> Result<AvailHeader, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(true);

		let block_hash = self.block_hash().await?;
		let block_header = self
			.client
			.rpc()
			.retry_on(Some(retry_on_error), Some(retry_on_none))
			.block_header(Some(block_hash))
			.await?;
		let Some(block_header) = block_header else {
			return Err("Failed to fetch best block header".into());
		};

		Ok(block_header)
	}

	pub async fn block(&self) -> Result<BlockWithJustifications, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(true);

		let block_hash = self.block_hash().await?;
		let block = self
			.client
			.rpc()
			.retry_on(Some(retry_on_error), Some(retry_on_none))
			.block(Some(block_hash))
			.await?;
		let Some(block) = block else {
			return Err("Best block not found".into());
		};

		Ok(block)
	}

	pub async fn block_nonce(&self, account_id: &AccountId) -> Result<u32, avail_rust_core::Error> {
		self.block_account_info(account_id).await.map(|v| v.nonce)
	}

	pub async fn block_balance(&self, account_id: &AccountId) -> Result<AccountData, avail_rust_core::Error> {
		self.block_account_info(account_id).await.map(|x| x.data)
	}

	pub async fn block_account_info(&self, account_id: &AccountId) -> Result<AccountInfo, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let at = self.block_hash().await?;
		self.client
			.rpc()
			.retry_on(Some(retry_on_error), None)
			.account_info(account_id, at)
			.await
	}

	pub async fn block_info(&self) -> Result<BlockRef, avail_rust_core::Error> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		self.client
			.rpc()
			.retry_on(Some(retry_on_error), None)
			.block_info(false)
			.await
	}

	pub async fn block_hash(&self) -> Result<H256, avail_rust_core::Error> {
		self.block_info().await.map(|x| x.hash)
	}

	pub async fn block_height(&self) -> Result<u32, avail_rust_core::Error> {
		self.block_info().await.map(|x| x.height)
	}
}
