//! High-level Avail client combining RPC access with helper APIs for blocks and transactions.

use super::online_client::OnlineClient;
use crate::{
	BlockState, Error, UserError, avail,
	block_api::BlockApi,
	clients::utils::{with_retry_on_error, with_retry_on_error_and_none},
	submission_api::SubmittedTransaction,
	subxt_rpcs::RpcClient,
	subxt_signer::sr25519::Keypair,
	transaction_api::TransactionApi,
	transaction_options::Options,
};
use avail::{
	balances::types::AccountData,
	system::{storage as SystemStorage, types::AccountInfo},
};
use avail_rust_core::{
	AccountId, AccountIdLike, AvailHeader, BlockInfo, H256, HashNumber, StorageMap,
	grandpa::GrandpaJustification,
	rpc::{self, BlockPhaseEvent, Error as RpcError, ExtrinsicInfo, LegacyBlock, runtime_api},
	types::{
		HashString,
		metadata::{ChainInfo, HashStringNumber},
		substrate::{FeeDetails, RuntimeDispatchInfo},
	},
};
use codec::Decode;
#[cfg(feature = "tracing")]
use tracing_subscriber::util::TryInitError;

/// Primary entry point used throughout the SDK to interact with the node.
#[derive(Clone)]
pub struct Client {
	online_client: OnlineClient,
	pub rpc_client: RpcClient,
}

impl Client {
	#[cfg(feature = "reqwest")]
	/// Connects to an endpoint and returns a ready-to-use client.
	///
	/// Retries automatically if the `reqwest` feature is enabled and `retry` defaults to `true`.
	///
	/// # Errors
	/// Returns `Err(Error)` if the transport cannot be initialized or the handshake fails.
	pub async fn new(endpoint: &str) -> Result<Client, Error> {
		Self::new_ext(endpoint, true).await
	}

	#[cfg(feature = "reqwest")]
	/// Connects to an endpoint; set `retry` to `false` if you prefer failing fast during startup.
	///
	/// Internally wraps [`with_retry_on_error`] so transient errors (network hiccups, etc.) are retried
	/// when `retry` is `true`.
	pub async fn new_ext(endpoint: &str, retry: bool) -> Result<Client, Error> {
		use super::reqwest_client::ReqwestClient;

		let op = async || -> Result<Client, Error> {
			let rpc_client = ReqwestClient::new(endpoint);
			let rpc_client = RpcClient::new(rpc_client);

			Self::from_rpc_client(rpc_client).await.map_err(|e| e.into())
		};

		with_retry_on_error(op, retry).await
	}

	/// Builds a client from an existing RPC transport.
	///
	/// # Errors
	/// Propagates any failure returned by [`OnlineClient::new`].
	pub async fn from_rpc_client(rpc_client: RpcClient) -> Result<Client, RpcError> {
		let online_client = OnlineClient::new(&rpc_client).await?;
		Self::from_components(rpc_client, online_client).await
	}

	/// Wraps pre-built components into a handy client handle.
	///
	/// Useful when creating mock transports for testing.
	pub async fn from_components(rpc_client: RpcClient, online_client: OnlineClient) -> Result<Client, RpcError> {
		Ok(Self { online_client, rpc_client })
	}

	#[cfg(feature = "tracing")]
	/// Initializes tracing in plain text or JSON format.
	///
	/// Calling this more than once will return an error from `tracing_subscriber`.
	pub fn init_tracing(json_format: bool) -> Result<(), TryInitError> {
		use tracing_subscriber::util::SubscriberInitExt;

		let builder = tracing_subscriber::fmt::SubscriberBuilder::default();
		if json_format {
			let builder = builder.json();
			builder.finish().try_init()
		} else {
			builder.finish().try_init()
		}
	}

	/// Hands back the underlying `OnlineClient` for advanced uses.
	pub fn online_client(&self) -> OnlineClient {
		self.online_client.clone()
	}

	/// Gives you a helper for crafting and sending transactions.
	///
	/// The returned [`TransactionApi`] clones this client and issues RPCs lazily.
	pub fn tx(&self) -> TransactionApi {
		TransactionApi(self.clone())
	}

	/// Builds a block helper rooted at the height or hash you pass in.
	///
	/// See [`BlockApi`] for available views (transactions, events, raw extrinsics).
	pub fn block(&self, block_id: impl Into<HashStringNumber>) -> BlockApi {
		BlockApi::new(self.clone(), block_id)
	}

	/// Provides low-level RPC helpers when you need finer control.
	///
	/// The returned [`ChainApi`] exposes retry toggles and raw RPC wrappers.
	pub fn chain(&self) -> ChainApi {
		ChainApi::new(self.clone())
	}

	/// Provides quick access to the best (head) block view.
	///
	/// Use the returned [`Best`] helper to fetch head metadata or block info repeatedly.
	pub fn best(&self) -> Best {
		Best::new(self.clone())
	}

	/// Provides quick access to finalized block information.
	///
	/// The returned [`Finalized`] helper exposes convenience methods similar to [`Best`].
	pub fn finalized(&self) -> Finalized {
		Finalized::new(self.clone())
	}

	/// Reports whether automatic retries are currently enabled.
	pub fn is_global_retries_enabled(&self) -> bool {
		self.online_client.is_global_retries_enabled()
	}

	/// Turns automatic retries on or off for new requests.
	pub fn set_global_retries_enabled(&self, value: bool) {
		self.online_client.set_global_retries_enabled(value);
	}
}

/// Low-level RPC surface with fine-grained retry controls.
pub struct ChainApi {
	client: Client,
	retry_on_error: Option<bool>,
	retry_on_none: Option<bool>,
}
impl ChainApi {
	/// Creates a helper for low-level RPC calls.
	pub fn new(client: Client) -> Self {
		Self { client, retry_on_error: None, retry_on_none: None }
	}

	/// Lets you decide if upcoming calls retry on errors or missing data.
	///
	/// - `error`: overrides whether transport errors are retried (defaults to the client's global flag).
	/// - `none`: when `Some(true)`, RPCs returning `None` (e.g., missing storage) will also be retried.
	pub fn retry_on(mut self, error: Option<bool>, none: Option<bool>) -> Self {
		self.retry_on_error = error;
		self.retry_on_none = none;
		self
	}

	/// Fetches a block hash for the given height when available.
	///
	/// # Returns
	/// - `Ok(Some(H256))` when the chain knows about the requested height.
	/// - `Ok(None)` when the block does not exist (and `retry_on_none` is `false`).
	/// - `Err(RpcError)` when the underlying RPC call fails.
	pub async fn block_hash(&self, block_height: Option<u32>) -> Result<Option<H256>, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::chain::get_block_hash(&self.client.rpc_client, block_height).await };
		with_retry_on_error_and_none(f, retry_on_error, retry_on_none).await
	}

	/// Grabs a block header by hash or height.
	///
	/// # Returns
	/// - `Ok(Some(AvailHeader))` when the header exists.
	/// - `Ok(None)` when the header is missing (and `retry_on_none` is `false`).
	/// - `Err(Error)` when conversions or RPC calls fail.
	pub async fn block_header(&self, at: Option<impl Into<HashStringNumber>>) -> Result<Option<AvailHeader>, Error> {
		async fn inner(r: &ChainApi, at: Option<HashStringNumber>) -> Result<Option<AvailHeader>, Error> {
			let retry_on_error = r.retry_on_error.unwrap_or(true);
			let retry_on_none = r.retry_on_none.unwrap_or(false);

			let at = if let Some(at) = at {
				let at: HashNumber = at.try_into().map_err(UserError::Other)?;
				let at = match at {
					HashNumber::Hash(h) => h,
					HashNumber::Number(n) => r
						.block_hash(Some(n))
						.await?
						.ok_or(UserError::Other("No block bound for that block height".into()))?,
				};
				Some(at)
			} else {
				None
			};

			let f = || async move { rpc::chain::get_header(&r.client.rpc_client, at).await };
			Ok(with_retry_on_error_and_none(f, retry_on_error, retry_on_none).await?)
		}

		inner(self, at.map(|x| x.into())).await
	}

	/// Retrieves the full legacy block when you need the old format.
	///
	/// Return semantics match [`ChainApi::block_hash`] regarding retries and `None` handling.
	pub async fn legacy_block(&self, at: Option<H256>) -> Result<Option<LegacyBlock>, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::chain::get_block(&self.client.rpc_client, at).await };
		with_retry_on_error_and_none(f, retry_on_error, retry_on_none).await
	}

	/// Looks up an account nonce at a particular block.
	///
	/// # Errors
	/// Returns `Err(Error)` when the account id cannot be parsed or the RPC call fails.
	pub async fn block_nonce(
		&self,
		account_id: impl Into<AccountIdLike>,
		at: impl Into<HashStringNumber>,
	) -> Result<u32, Error> {
		self.account_info(account_id, at).await.map(|x| x.nonce)
	}

	/// Returns the latest account nonce as seen by the node.
	///
	/// # Errors
	/// Returns `Err(Error)` when the account identifier is invalid or the RPC call fails.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		async fn inner(r: &ChainApi, account_id: AccountIdLike) -> Result<u32, Error> {
			let retry_on_error = r.retry_on_error.unwrap_or_else(|| r.client.is_global_retries_enabled());
			let account_id: AccountId = account_id.try_into().map_err(UserError::Other)?;

			let a = &account_id;
			let f =
				|| async move { rpc::system::account_next_index(&r.client.rpc_client, &std::format!("{}", a)).await };

			Ok(with_retry_on_error(f, retry_on_error).await?)
		}

		inner(self, account_id.into()).await
	}

	/// Reports the free balance for an account at a specific block.
	///
	/// Errors mirror [`ChainApi::account_info`].
	pub async fn account_balance(
		&self,
		account_id: impl Into<AccountIdLike>,
		at: impl Into<HashStringNumber>,
	) -> Result<AccountData, Error> {
		self.account_info(account_id, at).await.map(|x| x.data)
	}

	/// Fetches the full account record (nonce, balances, …) at a given block.
	///
	/// # Errors
	/// Returns `Err(Error)` when the account identifier or block id cannot be converted, the block is
	/// missing, or the RPC call fails.
	pub async fn account_info(
		&self,
		account_id: impl Into<AccountIdLike>,
		at: impl Into<HashStringNumber>,
	) -> Result<AccountInfo, Error> {
		async fn inner(r: &ChainApi, account_id: AccountIdLike, at: HashStringNumber) -> Result<AccountInfo, Error> {
			let retry_on_error = r.retry_on_error.unwrap_or_else(|| r.client.is_global_retries_enabled());

			let account_id: AccountId = account_id.try_into().map_err(UserError::Other)?;
			let block_id: HashNumber = at.try_into().map_err(UserError::Other)?;
			let at = match block_id {
				HashNumber::Hash(h) => h,
				HashNumber::Number(n) => r
					.block_hash(Some(n))
					.await?
					.ok_or(UserError::Other("No block bound for that block height".into()))?,
			};

			let a = &account_id;
			let f = || async move {
				SystemStorage::Account::fetch(&r.client.rpc_client, a, Some(at))
					.await
					.map(|x| x.unwrap_or_default())
			};

			Ok(with_retry_on_error(f, retry_on_error).await?)
		}

		inner(self, account_id.into(), at.into()).await
	}

	/// Tells you if a block is pending, finalized, or missing.
	///
	/// # Returns
	/// Distinguishes between [`BlockState::Included`], [`BlockState::Finalized`], [`BlockState::Discarded`],
	/// and [`BlockState::DoesNotExist`], depending on chain state.
	///
	/// # Errors
	/// Returns `Err(Error)` if the supplied identifier cannot be converted or RPC calls fail.
	pub async fn block_state(&self, block_id: impl Into<HashStringNumber>) -> Result<BlockState, Error> {
		async fn inner(r: &ChainApi, block_id: HashStringNumber) -> Result<BlockState, Error> {
			let block_id = HashNumber::try_from(block_id).map_err(UserError::Other)?;
			let chain_info = r.chain_info().await?;
			let n = match block_id {
				HashNumber::Hash(h) => {
					if h == chain_info.finalized_hash {
						return Ok(BlockState::Finalized);
					}

					if h == chain_info.best_hash {
						return Ok(BlockState::Included);
					}

					let Some(n) = r.block_height(h).await? else {
						return Ok(BlockState::DoesNotExist);
					};

					let Some(block_hash) = r.block_hash(Some(n)).await? else {
						return Ok(BlockState::DoesNotExist);
					};

					if block_hash != h {
						return Ok(BlockState::Discarded);
					}

					n
				},
				HashNumber::Number(n) => n,
			};

			if n > chain_info.best_height {
				return Ok(BlockState::DoesNotExist);
			}

			if n > chain_info.finalized_height {
				return Ok(BlockState::Included);
			}

			return Ok(BlockState::Finalized);
		}

		inner(self, block_id.into()).await
	}

	/// Converts a block hash into its block height when possible.
	///
	/// Returns `Ok(None)` when the node does not recognise the hash and `retry_on_none` is `false`.
	pub async fn block_height(&self, at: impl Into<HashString>) -> Result<Option<u32>, Error> {
		async fn inner(r: &ChainApi, at: HashString) -> Result<Option<u32>, Error> {
			let retry_on_error = r.retry_on_error.unwrap_or_else(|| r.client.is_global_retries_enabled());
			let retry_on_none = r.retry_on_none.unwrap_or(false);

			let at: H256 = at.try_into().map_err(|x| UserError::Other(x))?;
			let f = || async move { rpc::system::get_block_number(&r.client.rpc_client, at).await };
			Ok(with_retry_on_error_and_none(f, retry_on_error, retry_on_none).await?)
		}

		inner(self, at.into()).await
	}

	/// Returns the latest block info, either best or finalized.
	///
	/// This is a thin wrapper over `system_latest_block_info` with retry support.
	pub async fn block_info(&self, use_best_block: bool) -> Result<BlockInfo, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());
		let f = || async move { rpc::system::latest_block_info(&self.client.rpc_client, use_best_block).await };
		with_retry_on_error(f, retry_on_error).await
	}

	/// Quick snapshot of both the best and finalized heads.
	///
	/// Mirrors `system_latest_chain_info`, exposing height/hash pairs for head and finalized.
	pub async fn chain_info(&self) -> Result<ChainInfo, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let f = || async move { rpc::system::latest_chain_info(&self.client.rpc_client).await };
		with_retry_on_error(f, retry_on_error).await
	}

	/// Submits a signed extrinsic and gives you the transaction hash.
	///
	/// # Errors
	/// Returns `Err(RpcError)` when the node rejects the extrinsic or the RPC transport fails.
	pub async fn submit(&self, tx: &avail_rust_core::GenericExtrinsic<'_>) -> Result<H256, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let encoded = tx.encode();
		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signature {
			if let avail_rust_core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "tx", "Submitting Transaction. Address: {}, Nonce: {}, App Id: {}", account_id, signed.tx_extra.nonce, signed.tx_extra.app_id);
			}
		}

		let enc_slice = encoded.as_slice();
		let f = || async move { rpc::author::submit_extrinsic(&self.client.rpc_client, enc_slice).await };
		let tx_hash = with_retry_on_error(f, retry_on_error).await?;

		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signature {
			if let avail_rust_core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "tx", "Transaction Submitted.  Address: {}, Nonce: {}, App Id: {}, Tx Hash: {:?},", account_id, signed.tx_extra.nonce, signed.tx_extra.app_id, tx_hash);
			}
		}

		Ok(tx_hash)
	}

	/// Signs an already prepared payload with the provided keypair.
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

	/// Builds a payload from a call and signs it with sensible defaults.
	///
	/// # Errors
	/// Returns `Err(Error)` when option refinement fails (e.g., fetching account info) or signing fails.
	pub async fn sign_call<'a>(
		&self,
		signer: &Keypair,
		tx_call: &'a avail_rust_core::ExtrinsicCall,
		options: Options,
	) -> Result<avail_rust_core::GenericExtrinsic<'a>, Error> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

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

	/// Signs the payload and submits it in one step.
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

	/// Signs a call, submits it, and hands back a tracker you can poll.
	///
	/// # Returns
	/// - `Ok(SubmittedTransaction)` containing the transaction hash and refined options for later
	///   receipt queries.
	/// - `Err(Error)` when option refinement, signing, or submission fails.
	pub async fn sign_and_submit_call(
		&self,
		signer: &Keypair,
		tx_call: &avail_rust_core::ExtrinsicCall,
		options: Options,
	) -> Result<SubmittedTransaction, Error> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

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

	/// Runs a `state_call` and returns the raw response string.
	///
	/// Retries according to the helper's configuration before surfacing `Err(RpcError)`.
	pub async fn state_call(&self, method: &str, data: &[u8], at: Option<H256>) -> Result<String, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let f = || async move { rpc::state::call(&self.client.rpc_client, method, data, at).await };
		with_retry_on_error(f, retry_on_error).await
	}

	/// Downloads runtime metadata as bytes.
	pub async fn state_get_metadata(&self, at: Option<H256>) -> Result<Vec<u8>, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let f = || async move { rpc::state::get_metadata(&self.client.rpc_client, at).await };
		with_retry_on_error(f, retry_on_error).await
	}

	/// Reads a storage entry, returning the raw bytes if present.
	pub async fn state_get_storage(&self, key: &str, at: Option<H256>) -> Result<Option<Vec<u8>>, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let f = || async move { rpc::state::get_storage(&self.client.rpc_client, key, at).await };
		with_retry_on_error(f, retry_on_error).await
	}

	/// Lists storage keys under a prefix, one page at a time.
	pub async fn state_get_keys_paged(
		&self,
		prefix: Option<&str>,
		count: u32,
		start_key: Option<&str>,
		at: Option<H256>,
	) -> Result<Vec<String>, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let f =
			|| async move { rpc::state::get_keys_paged(&self.client.rpc_client, prefix, count, start_key, at).await };

		with_retry_on_error(f, retry_on_error).await
	}

	/// Collects extrinsics from a block using the provided filters.
	///
	/// # Errors
	/// Returns `Err(Error)` when the block id cannot be decoded or the RPC request fails.
	pub async fn system_fetch_extrinsics(
		&self,
		block_id: impl Into<HashStringNumber>,
		opts: rpc::ExtrinsicOpts,
	) -> Result<Vec<ExtrinsicInfo>, Error> {
		async fn inner(
			r: &ChainApi,
			block_id: HashStringNumber,
			opts: &rpc::ExtrinsicOpts,
		) -> Result<Vec<ExtrinsicInfo>, Error> {
			let retry_on_error = r.retry_on_error.unwrap_or_else(|| r.client.is_global_retries_enabled());

			let block_id: HashNumber = block_id.try_into().map_err(UserError::Decoding)?;
			let f = || async move { rpc::system::fetch_extrinsics_v1(&r.client.rpc_client, block_id, opts).await };
			with_retry_on_error(f, retry_on_error).await.map_err(|e| e.into())
		}

		inner(&self, block_id.into(), &opts).await
	}

	/// Pulls events for a block with optional filtering.
	///
	/// # Errors
	/// Returns `Err(Error)` when the block id cannot be resolved or the RPC call fails.
	pub async fn system_fetch_events(
		&self,
		at: impl Into<HashStringNumber>,
		opts: rpc::EventOpts,
	) -> Result<Vec<BlockPhaseEvent>, Error> {
		async fn inner(
			r: &ChainApi,
			block_id: HashStringNumber,
			opts: &rpc::EventOpts,
		) -> Result<Vec<BlockPhaseEvent>, Error> {
			let retry_on_error = r.retry_on_error.unwrap_or_else(|| r.client.is_global_retries_enabled());

			let block_id: HashNumber = block_id.try_into().map_err(UserError::Decoding)?;
			let at = match block_id {
				HashNumber::Hash(x) => x,
				HashNumber::Number(x) => {
					let hash = r.block_hash(Some(x)).await?;
					hash.ok_or(RpcError::ExpectedData("Failed to fetch block height".into()))?
				},
			};

			let f = || async move { rpc::system::fetch_events_v1(&r.client.rpc_client, at, opts).await };
			with_retry_on_error(f, retry_on_error).await.map_err(|e| e.into())
		}

		inner(self, at.into(), &opts).await
	}

	/// Fetches a binary GRANDPA justification for the given block number.
	///
	/// # Returns
	/// - `Ok(Some(GrandpaJustification))` when a justification is present.
	/// - `Ok(None)` when the runtime returns no justification.
	/// - `Err(RpcError)` if decoding the response or the RPC call fails.
	pub async fn grandpa_block_justification(&self, at: u32) -> Result<Option<GrandpaJustification>, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let f = || async move { rpc::grandpa::block_justification(&self.client.rpc_client, at).await };
		let result = with_retry_on_error(f, retry_on_error).await?;

		let Some(result) = result else {
			return Ok(None);
		};

		let justification = const_hex::decode(result.trim_start_matches("0x"))
			.map_err(|x| RpcError::MalformedResponse(x.to_string()))?;

		let justification = GrandpaJustification::decode(&mut justification.as_slice());
		let justification = justification.map_err(|e| RpcError::MalformedResponse(e.to_string()))?;
		Ok(Some(justification))
	}

	/// Fetches a JSON GRANDPA justification for the given block number.
	///
	/// Return semantics mirror [`ChainApi::grandpa_block_justification`], but the payload is parsed
	/// from JSON.
	pub async fn grandpa_block_justification_json(&self, at: u32) -> Result<Option<GrandpaJustification>, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let f = || async move { rpc::grandpa::block_justification_json(&self.client.rpc_client, at).await };
		let result = with_retry_on_error(f, retry_on_error).await?;

		let Some(result) = result else {
			return Ok(None);
		};

		let justification: Result<GrandpaJustification, _> = serde_json::from_str(result.as_str());
		let justification: GrandpaJustification =
			justification.map_err(|e| RpcError::MalformedResponse(e.to_string()))?;
		Ok(Some(justification))
	}

	/// Calls into the runtime API and decodes the answer for you.
	///
	/// This helper does not apply retries; use [`ChainApi::retry_on`] beforehand if desired.
	pub async fn call<T: codec::Decode>(&self, method: &str, data: &[u8], at: Option<H256>) -> Result<T, RpcError> {
		runtime_api::call_raw(&self.client.rpc_client, method, data, at).await
	}

	/// Estimates fees for a signed extrinsic.
	pub async fn transaction_payment_query_info(
		&self,
		extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, RpcError> {
		runtime_api::api_transaction_payment_query_info(&self.client.rpc_client, extrinsic, at).await
	}

	/// Breaks down the fee details for a signed extrinsic.
	pub async fn transaction_payment_query_fee_details(
		&self,
		extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, RpcError> {
		runtime_api::api_transaction_payment_query_fee_details(&self.client.rpc_client, extrinsic, at).await
	}

	/// Estimates fees for an unsigned call payload.
	pub async fn transaction_payment_query_call_info(
		&self,
		call: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, RpcError> {
		runtime_api::api_transaction_payment_query_call_info(&self.client.rpc_client, call, at).await
	}

	/// Breaks down the fee details for an unsigned call payload.
	pub async fn transaction_payment_query_call_fee_details(
		&self,
		call: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, RpcError> {
		runtime_api::api_transaction_payment_query_call_fee_details(&self.client.rpc_client, call, at).await
	}

	pub async fn blob_submit_blob(&self, metadata_signed_transaction: &[u8], blob: &[u8]) -> Result<(), Error> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let f =
			|| async move { rpc::blob::submit_blob(&self.client.rpc_client, metadata_signed_transaction, blob).await };

		Ok(with_retry_on_error(f, retry_on_error).await?)
	}
}

/// Helper bound to the chain's best (head) block view.
pub struct Best {
	client: Client,
	retry_on_error: Option<bool>,
}
impl Best {
	/// Builds a helper focused on the best (head) block.
	pub fn new(client: Client) -> Self {
		Self { client, retry_on_error: None }
	}

	/// Lets you toggle automatic retries for the calls that follow.
	pub fn retry_on(mut self, error: Option<bool>) -> Self {
		self.retry_on_error = error;
		self
	}

	/// Returns the current best block header.
	///
	/// # Errors
	/// Returns `Err(Error)` when the head hash cannot be fetched or the header lookup fails.
	pub async fn block_header(&self) -> Result<AvailHeader, Error> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let block_hash = self.block_hash().await?;
		let block_header = self
			.client
			.chain()
			.retry_on(Some(retry_on_error), Some(true))
			.block_header(Some(block_hash))
			.await?;
		let Some(block_header) = block_header else {
			return Err(RpcError::ExpectedData("Failed to fetch best block header".into()).into());
		};

		Ok(block_header)
	}

	/// Gives you a block handle for the best block.
	pub async fn block(&self) -> Result<BlockApi, Error> {
		let block_hash = self.block_hash().await?;
		Ok(BlockApi::new(self.client.clone(), block_hash))
	}

	/// Returns height and hash for the best block.
	///
	/// Equivalent to `chain().block_info(true)` but respecting this helper's retry setting.
	pub async fn block_info(&self) -> Result<BlockInfo, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		self.client
			.chain()
			.retry_on(Some(retry_on_error), Some(true))
			.block_info(true)
			.await
	}

	/// Returns the hash of the best block.
	pub async fn block_hash(&self) -> Result<H256, RpcError> {
		self.block_info().await.map(|x| x.hash)
	}

	/// Returns the height of the best block.
	pub async fn block_height(&self) -> Result<u32, RpcError> {
		self.block_info().await.map(|x| x.height)
	}

	/// Loads the legacy block view for the best block.
	///
	/// # Errors
	/// Returns `Err(RpcError::ExpectedData)` when the node reports no legacy block for the head.
	pub async fn legacy_block(&self) -> Result<LegacyBlock, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let block_hash = self.block_hash().await?;
		let block = self
			.client
			.chain()
			.retry_on(Some(retry_on_error), Some(true))
			.legacy_block(Some(block_hash))
			.await?;
		let Some(block) = block else {
			return Err(RpcError::ExpectedData("Failed to fetch latest legacy block".into()));
		};

		Ok(block)
	}

	/// Returns the latest nonce for the account at the best block.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		self.account_info(account_id).await.map(|v| v.nonce)
	}

	/// Returns the account balances at the best block.
	pub async fn account_balance(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountData, Error> {
		self.account_info(account_id).await.map(|x| x.data)
	}

	/// Returns the full account record at the best block.
	///
	/// # Errors
	/// Mirrors [`ChainApi::account_info`].
	pub async fn account_info(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountInfo, Error> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let at = self.block_hash().await?;
		self.client
			.chain()
			.retry_on(Some(retry_on_error), Some(true))
			.account_info(account_id, at)
			.await
	}
}

/// Helper bound to the chain's latest finalized block view.
pub struct Finalized {
	client: Client,
	retry_on_error: Option<bool>,
}

impl Finalized {
	/// Builds a helper focused on finalized blocks.
	pub fn new(client: Client) -> Self {
		Self { client, retry_on_error: None }
	}

	/// Lets you toggle automatic retries for the calls that follow.
	pub fn retry_on(mut self, error: Option<bool>) -> Self {
		self.retry_on_error = error;
		self
	}

	/// Returns the latest finalized block header.
	///
	/// # Errors
	/// Returns `Err(Error)` if fetching the hash or header fails.
	pub async fn block_header(&self) -> Result<AvailHeader, Error> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let block_hash = self.block_hash().await?;
		let block_header = self
			.client
			.chain()
			.retry_on(Some(retry_on_error), Some(true))
			.block_header(Some(block_hash))
			.await?;
		let Some(block_header) = block_header else {
			return Err(RpcError::ExpectedData("Failed to fetch finalized block header".into()).into());
		};

		Ok(block_header)
	}

	/// Gives you a block handle for the latest finalized block.
	pub async fn block(&self) -> Result<BlockApi, Error> {
		let block_hash = self.block_hash().await?;
		Ok(BlockApi::new(self.client.clone(), block_hash))
	}

	/// Returns height and hash for the latest finalized block.
	///
	/// Equivalent to `chain().block_info(false)` with retry controls preserved.
	pub async fn block_info(&self) -> Result<BlockInfo, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		self.client
			.chain()
			.retry_on(Some(retry_on_error), self.retry_on_error)
			.block_info(false)
			.await
	}

	/// Returns the hash of the latest finalized block.
	pub async fn block_hash(&self) -> Result<H256, RpcError> {
		self.block_info().await.map(|x| x.hash)
	}

	/// Returns the height of the latest finalized block.
	pub async fn block_height(&self) -> Result<u32, RpcError> {
		self.block_info().await.map(|x| x.height)
	}

	/// Loads the legacy block view for the latest finalized block.
	///
	/// Returns `Err(RpcError::ExpectedData)` when the node does not provide a legacy block.
	pub async fn legacy_block(&self) -> Result<LegacyBlock, RpcError> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let block_hash = self.block_hash().await?;
		let block = self
			.client
			.chain()
			.retry_on(Some(retry_on_error), Some(true))
			.legacy_block(Some(block_hash))
			.await?;
		let Some(block) = block else {
			return Err(RpcError::ExpectedData("Failed to fetch latest legacy block".into()));
		};

		Ok(block)
	}

	/// Returns the latest finalized nonce for the account.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		self.account_info(account_id).await.map(|v| v.nonce)
	}

	/// Returns account balances from the latest finalized block.
	pub async fn account_balance(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountData, Error> {
		self.account_info(account_id).await.map(|x| x.data)
	}

	/// Returns the full account record from the latest finalized block.
	///
	/// Errors mirror [`ChainApi::account_info`].
	pub async fn account_info(&self, account_id: impl Into<AccountIdLike>) -> Result<AccountInfo, Error> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let at = self.block_hash().await?;
		self.client
			.chain()
			.retry_on(Some(retry_on_error), Some(true))
			.account_info(account_id, at)
			.await
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
