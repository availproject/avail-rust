use super::online_client::OnlineClient;
use crate::{
	BlockState, Error, UserError, avail,
	block::Block,
	clients::utils::{with_retry_on_error, with_retry_on_error_and_none},
	extrinsic::SubmittedTransaction,
	subxt_rpcs::RpcClient,
	subxt_signer::sr25519::Keypair,
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
	rpc::{self, BlockPhaseEvent, BlockWithJustifications, Error as RpcError, ExtrinsicInfo, runtime_api},
	types::{
		metadata::HashStringNumber,
		substrate::{FeeDetails, RuntimeDispatchInfo},
	},
};
use codec::Decode;

#[derive(Clone)]
pub struct Client {
	online_client: OnlineClient,
	pub rpc_client: RpcClient,
}

impl Client {
	#[cfg(feature = "reqwest")]
	pub async fn new(endpoint: &str) -> Result<Client, Error> {
		use super::reqwest_client::ReqwestClient;

		let op = async || -> Result<Client, Error> {
			let rpc_client = ReqwestClient::new(endpoint);
			let rpc_client = RpcClient::new(rpc_client);

			Self::new_rpc_client(rpc_client).await.map_err(|e| e.into())
		};

		with_retry_on_error(op, true, "").await
	}

	pub async fn new_rpc_client(rpc_client: RpcClient) -> Result<Client, RpcError> {
		let online_client = OnlineClient::new(&rpc_client).await?;

		Self::new_custom(rpc_client, online_client).await
	}

	pub async fn new_custom(rpc_client: RpcClient, online_client: OnlineClient) -> Result<Client, RpcError> {
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

	pub fn online_client(&self) -> OnlineClient {
		self.online_client.clone()
	}

	pub fn block(&self, block_id: impl Into<HashStringNumber>) -> Block {
		Block::new(self.clone(), block_id)
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

	pub async fn block_hash(&self, block_height: Option<u32>) -> Result<Option<H256>, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::chain::get_block_hash(&self.client.rpc_client, block_height).await };
		with_retry_on_error_and_none(f, retry_on_error, retry_on_none, "").await
	}

	pub async fn block_header(&self, at: Option<H256>) -> Result<Option<AvailHeader>, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::chain::get_header(&self.client.rpc_client, at).await };
		with_retry_on_error_and_none(f, retry_on_error, retry_on_none, "").await
	}

	pub async fn block_with_justification(
		&self,
		at: Option<H256>,
	) -> Result<Option<BlockWithJustifications>, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::chain::get_block(&self.client.rpc_client, at).await };
		with_retry_on_error_and_none(f, retry_on_error, retry_on_none, "").await
	}

	// Nonce
	pub async fn account_nonce(&self, account_id: &AccountId) -> Result<u32, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move {
			rpc::system::account_next_index(&self.client.rpc_client, &std::format!("{}", account_id)).await
		};
		with_retry_on_error(f, retry_on_error, "").await
	}

	pub async fn block_nonce(&self, account_id: &AccountId, block_hash: H256) -> Result<u32, RpcError> {
		self.account_info(account_id, block_hash).await.map(|x| x.nonce)
	}

	// Balance
	pub async fn account_balance(&self, account_id: &AccountId, at: H256) -> Result<AccountData, RpcError> {
		self.account_info(account_id, at).await.map(|x| x.data)
	}

	// Account Info (nonce, balance, ...)
	pub async fn account_info(&self, account_id: &AccountId, at: H256) -> Result<AccountInfo, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move {
			SystemStorage::Account::fetch(&self.client.rpc_client, account_id, Some(at))
				.await
				.map(|x| x.unwrap_or_default())
		};
		with_retry_on_error(f, retry_on_error, "").await
	}

	// Block State
	pub async fn block_state(&self, block_ref: BlockRef) -> Result<BlockState, RpcError> {
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
	pub async fn block_height(&self, at: H256) -> Result<Option<u32>, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::system::get_block_number(&self.client.rpc_client, at).await };
		with_retry_on_error_and_none(f, retry_on_error, retry_on_none, "").await
	}

	pub async fn block_info(&self, use_best_block: bool) -> Result<BlockRef, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move { rpc::system::latest_block_info(&self.client.rpc_client, use_best_block).await };
		with_retry_on_error(f, retry_on_error, "").await
	}

	// Sign and submit
	pub async fn submit(&self, tx: &avail_rust_core::GenericExtrinsic<'_>) -> Result<H256, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let encoded = tx.encode();
		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signature {
			if let avail_rust_core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "tx", "Submitting Transaction. Address: {}, Nonce: {}, App Id: {}", account_id, signed.tx_extra.nonce, signed.tx_extra.app_id);
			}
		}

		let enc_slice = encoded.as_slice();
		let f = || async move { rpc::author::submit_extrinsic(&self.client.rpc_client, enc_slice).await };
		let tx_hash = with_retry_on_error(f, retry_on_error, "").await?;

		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signature {
			if let avail_rust_core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "tx", "Transaction Submitted.  Address: {}, Nonce: {}, App Id: {}, Tx Hash: {:?},", account_id, signed.tx_extra.nonce, signed.tx_extra.app_id, tx_hash);
			}
		}

		Ok(tx_hash)
	}

	pub async fn sign_payload<'a>(
		&self,
		signer: &Keypair,
		tx_payload: avail_rust_core::ExtrinsicPayload<'a>,
	) -> avail_rust_core::GenericExtrinsic<'a> {
		use avail_rust_core::GenericExtrinsic;

		let account_id = signer.public_key().to_account_id();
		let signature = tx_payload.sign(signer);

		GenericExtrinsic::new(account_id, signature, tx_payload)
	}

	pub async fn sign_call<'a>(
		&self,
		signer: &Keypair,
		tx_call: &'a avail_rust_core::ExtrinsicCall,
		options: Options,
	) -> Result<avail_rust_core::GenericExtrinsic<'a>, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(&self.client, &account_id, retry_on_error).await?;

		let tx_extra = avail_rust_core::ExtrinsicExtra::from(&refined_options);
		let tx_additional = avail_rust_core::ExtrinsicAdditional {
			spec_version: self.client.online_client.spec_version(),
			tx_version: self.client.online_client.transaction_version(),
			genesis_hash: self.client.online_client.genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = avail_rust_core::ExtrinsicPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
		Ok(self.sign_payload(signer, tx_payload).await)
	}

	pub async fn sign_and_submit_payload(
		&self,
		signer: &Keypair,
		tx_payload: avail_rust_core::ExtrinsicPayload<'_>,
	) -> Result<H256, RpcError> {
		use avail_rust_core::GenericExtrinsic;

		let account_id = signer.public_key().to_account_id();
		let signature = tx_payload.sign(signer);
		let tx = GenericExtrinsic::new(account_id, signature, tx_payload);
		let tx_hash = self.submit(&tx).await?;

		Ok(tx_hash)
	}

	pub async fn sign_and_submit_call(
		&self,
		signer: &Keypair,
		tx_call: &avail_rust_core::ExtrinsicCall,
		options: Options,
	) -> Result<SubmittedTransaction, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(&self.client, &account_id, retry_on_error).await?;

		let tx_extra = avail_rust_core::ExtrinsicExtra::from(&refined_options);
		let tx_additional = avail_rust_core::ExtrinsicAdditional {
			spec_version: self.client.online_client.spec_version(),
			tx_version: self.client.online_client.transaction_version(),
			genesis_hash: self.client.online_client.genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = avail_rust_core::ExtrinsicPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
		let tx_hash = self.sign_and_submit_payload(signer, tx_payload).await?;

		let value = SubmittedTransaction::new(self.client.clone(), tx_hash, account_id, refined_options, tx_additional);
		Ok(value)
	}

	// Rest
	pub async fn state_call(&self, method: &str, data: &[u8], at: Option<H256>) -> Result<String, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move { rpc::state::call(&self.client.rpc_client, method, data, at).await };
		with_retry_on_error(f, retry_on_error, "").await
	}

	pub async fn state_get_metadata(&self, at: Option<H256>) -> Result<Vec<u8>, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move { rpc::state::get_metadata(&self.client.rpc_client, at).await };
		with_retry_on_error(f, retry_on_error, "").await
	}

	pub async fn state_get_storage(&self, key: &str, at: Option<H256>) -> Result<Option<Vec<u8>>, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f = || async move { rpc::state::get_storage(&self.client.rpc_client, key, at).await };
		with_retry_on_error(f, retry_on_error, "").await
	}

	pub async fn state_get_keys_paged(
		&self,
		prefix: Option<&str>,
		count: u32,
		start_key: Option<&str>,
		at: Option<H256>,
	) -> Result<Vec<String>, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let f =
			|| async move { rpc::state::get_keys_paged(&self.client.rpc_client, prefix, count, start_key, at).await };
		with_retry_on_error(f, retry_on_error, "").await
	}

	pub async fn system_fetch_extrinsics(
		&self,
		block_id: impl Into<HashStringNumber>,
		opts: rpc::ExtrinsicOpts,
	) -> Result<Vec<ExtrinsicInfo>, Error> {
		async fn inner(
			client: &Client,
			block_id: HashStringNumber,
			opts: &rpc::ExtrinsicOpts,
			retry_on_error: bool,
		) -> Result<Vec<ExtrinsicInfo>, Error> {
			let block_id: HashNumber = block_id.try_into().map_err(UserError::Decoding)?;
			let f = || async move { rpc::system::fetch_extrinsics_v1(&client.rpc_client, block_id, opts).await };
			with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
		}

		let retry_on_error = self.retry_on_error.unwrap_or(true);
		inner(&self.client, block_id.into(), &opts, retry_on_error).await
	}

	pub async fn system_fetch_events(
		&self,
		at: impl Into<HashStringNumber>,
		opts: rpc::EventOpts,
	) -> Result<Vec<BlockPhaseEvent>, Error> {
		async fn inner(
			c: &Rpc,
			block_id: HashStringNumber,
			opts: &rpc::EventOpts,
			retry_on_error: bool,
		) -> Result<Vec<BlockPhaseEvent>, Error> {
			let block_id: HashNumber = block_id.try_into().map_err(UserError::Decoding)?;
			let at = match block_id {
				HashNumber::Hash(x) => x,
				HashNumber::Number(x) => {
					let hash = c.block_hash(Some(x)).await?;
					hash.ok_or(RpcError::ExpectedData("Failed to fetch block height".into()))?
				},
			};

			let f = || async move { rpc::system::fetch_events_v1(&c.client.rpc_client, at, opts).await };
			with_retry_on_error(f, retry_on_error, "").await.map_err(|e| e.into())
		}

		let retry_on_error = self.retry_on_error.unwrap_or(true);
		inner(self, at.into(), &opts, retry_on_error).await
	}

	pub async fn grandpa_block_justification(&self, at: u32) -> Result<Option<GrandpaJustification>, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let f = || async move { rpc::grandpa::block_justification(&self.client.rpc_client, at).await };
		let result = with_retry_on_error(f, retry_on_error, "").await?;

		let Some(result) = result else {
			return Ok(None);
		};

		let justification = const_hex::decode(result.trim_start_matches("0x"))
			.map_err(|x| RpcError::MalformedResponse(x.to_string()))?;

		let justification = GrandpaJustification::decode(&mut justification.as_slice());
		let justification = justification.map_err(|e| RpcError::MalformedResponse(e.to_string()))?;
		Ok(Some(justification))
	}

	pub async fn grandpa_block_justification_json(&self, at: u32) -> Result<Option<GrandpaJustification>, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		let f = || async move { rpc::grandpa::block_justification_json(&self.client.rpc_client, at).await };
		let result = with_retry_on_error(f, retry_on_error, "").await?;

		let Some(result) = result else {
			return Ok(None);
		};

		let justification: Result<GrandpaJustification, _> = serde_json::from_str(result.as_str());
		let justification: GrandpaJustification =
			justification.map_err(|e| RpcError::MalformedResponse(e.to_string()))?;
		Ok(Some(justification))
	}

	pub async fn call<T: codec::Decode>(&self, method: &str, data: &[u8], at: Option<H256>) -> Result<T, RpcError> {
		runtime_api::call_raw(&self.client.rpc_client, method, data, at).await
	}

	pub async fn transaction_payment_query_info(
		&self,
		extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, RpcError> {
		runtime_api::api_transaction_payment_query_info(&self.client.rpc_client, extrinsic, at).await
	}

	pub async fn transaction_payment_query_fee_details(
		&self,
		extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, RpcError> {
		runtime_api::api_transaction_payment_query_fee_details(&self.client.rpc_client, extrinsic, at).await
	}

	pub async fn transaction_payment_query_call_info(
		&self,
		call: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, RpcError> {
		runtime_api::api_transaction_payment_query_call_info(&self.client.rpc_client, call, at).await
	}

	pub async fn transaction_payment_query_call_fee_details(
		&self,
		call: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, RpcError> {
		runtime_api::api_transaction_payment_query_call_fee_details(&self.client.rpc_client, call, at).await
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

	pub async fn block_header(&self) -> Result<AvailHeader, RpcError> {
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
			return Err(RpcError::ExpectedData("Failed to fetch best block header".into()));
		};

		Ok(block_header)
	}

	pub async fn block(&self) -> Result<Block, Error> {
		let block_hash = self.block_hash().await?;
		Ok(Block::new(self.client.clone(), block_hash))
	}

	pub async fn account_nonce(&self, account_id: &AccountId) -> Result<u32, RpcError> {
		self.account_info(account_id).await.map(|v| v.nonce)
	}

	pub async fn account_balance(&self, account_id: &AccountId) -> Result<AccountData, RpcError> {
		self.account_info(account_id).await.map(|x| x.data)
	}

	pub async fn account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let at = self.block_hash().await?;
		self.client
			.rpc()
			.retry_on(Some(retry_on_error), None)
			.account_info(account_id, at)
			.await
	}

	pub async fn block_info(&self) -> Result<BlockRef, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		self.client
			.rpc()
			.retry_on(Some(retry_on_error), None)
			.block_info(true)
			.await
	}

	pub async fn block_hash(&self) -> Result<H256, RpcError> {
		self.block_info().await.map(|x| x.hash)
	}

	pub async fn block_height(&self) -> Result<u32, RpcError> {
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

	pub async fn block_header(&self) -> Result<AvailHeader, RpcError> {
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
			return Err(RpcError::ExpectedData("Failed to fetch finalized block header".into()));
		};

		Ok(block_header)
	}

	pub async fn block(&self) -> Result<Block, Error> {
		let block_hash = self.block_hash().await?;
		Ok(Block::new(self.client.clone(), block_hash))
	}

	pub async fn account_nonce(&self, account_id: &AccountId) -> Result<u32, RpcError> {
		self.account_info(account_id).await.map(|v| v.nonce)
	}

	pub async fn account_balance(&self, account_id: &AccountId) -> Result<AccountData, RpcError> {
		self.account_info(account_id).await.map(|x| x.data)
	}

	pub async fn account_info(&self, account_id: &AccountId) -> Result<AccountInfo, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);

		let at = self.block_hash().await?;
		self.client
			.rpc()
			.retry_on(Some(retry_on_error), None)
			.account_info(account_id, at)
			.await
	}

	pub async fn block_info(&self) -> Result<BlockRef, RpcError> {
		let retry_on_error = self.retry_on_error.unwrap_or(true);
		self.client
			.rpc()
			.retry_on(Some(retry_on_error), None)
			.block_info(false)
			.await
	}

	pub async fn block_hash(&self) -> Result<H256, RpcError> {
		self.block_info().await.map(|x| x.hash)
	}

	pub async fn block_height(&self) -> Result<u32, RpcError> {
		self.block_info().await.map(|x| x.height)
	}
}

// use crate::{ExtrinsicEvent, ExtrinsicEvents, clients::Client, subxt_core::events::Phase};
// use avail_rust_core::{H256, HashNumber, decoded_events::RawEvent, rpc::system::fetch_events};

// pub const EVENTS_STORAGE_ADDRESS: &str = "0x26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7";

// #[derive(Debug, Clone)]
// pub struct HistoricalEvent {
// 	pub phase: Phase,
// 	// [Pallet_index, Variant_index, Event_data...]
// 	pub bytes: RawEvent,
// 	pub topics: Vec<H256>,
// }

// impl HistoricalEvent {
// 	pub fn emitted_index(&self) -> (u8, u8) {
// 		(self.bytes.pallet_index(), self.bytes.variant_index())
// 	}

// 	pub fn pallet_index(&self) -> u8 {
// 		self.bytes.pallet_index()
// 	}

// 	pub fn variant_index(&self) -> u8 {
// 		self.bytes.variant_index()
// 	}

// 	pub fn event_bytes(&self) -> &[u8] {
// 		&self.bytes.0
// 	}

// 	pub fn event_data(&self) -> &[u8] {
// 		self.bytes.event_data()
// 	}
// }

// #[derive(Clone)]
// pub struct EventClient {
// 	client: Client,
// }

// impl EventClient {
// 	pub fn new(client: Client) -> Self {
// 		Self { client }
// 	}

// 	/// Use this function in case where `transaction_events` or `block_events` do not work.
// 	/// Both mentioned functions require the runtime to have a specific runtime api available which
// 	/// older blocks (runtime) do not have.
// 	pub async fn historical_block_events(&self, at: H256) -> Result<Vec<HistoricalEvent>, RpcError> {
// 		use crate::{config::AvailConfig, subxt_core::events::Events};

// 		let entries = self
// 			.client
// 			.rpc()
// 			.state_get_storage(EVENTS_STORAGE_ADDRESS, Some(at))
// 			.await?;
// 		let Some(event_bytes) = entries else {
// 			return Ok(Vec::new());
// 		};

// 		let mut result: Vec<HistoricalEvent> = Vec::with_capacity(5);
// 		let raw_events = Events::<AvailConfig>::decode_from(event_bytes, self.client.online_client().metadata());
// 		for raw in raw_events.iter() {
// 			let Ok(raw) = raw else {
// 				continue;
// 			};
// 			let mut bytes: Vec<u8> = Vec::with_capacity(raw.field_bytes().len() + 2);
// 			bytes.push(raw.pallet_index());
// 			bytes.push(raw.variant_index());
// 			bytes.append(&mut raw.field_bytes().to_vec());

// 			let Ok(bytes) = RawEvent::try_from(bytes) else {
// 				continue;
// 			};

// 			let value = HistoricalEvent { phase: raw.phase(), bytes, topics: raw.topics().to_vec() };
// 			result.push(value);
// 		}

// 		Ok(result)
// 	}
// }
