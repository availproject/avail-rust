use crate::{
	BlockState, Client, Error, UserError, avail,
	submission_api::SubmittedTransaction,
	subxt_signer::sr25519::Keypair,
	transaction_options::Options,
	utils::{with_retry_on_error, with_retry_on_error_and_none},
};
use avail::{
	balances::types::AccountData,
	system::{storage as SystemStorage, types::AccountInfo},
};
use avail_rust_core::{
	AccountId, AccountIdLike, AvailHeader, BlockInfo, H256, HashNumber, StorageMap,
	ext::subxt_rpcs::client::RpcParams,
	grandpa::GrandpaJustification,
	rpc::{
		self, BlockPhaseEvent, Error as RpcError, ExtrinsicInfo, LegacyBlock,
		kate::{BlockLength, Cell, GCellBlock, GDataProof, GMultiProof, GRow, ProofResponse},
		runtime_api,
	},
	types::{
		HashString,
		metadata::{ChainInfo, HashStringNumber},
		substrate::{FeeDetails, RuntimeDispatchInfo},
	},
};
use codec::Decode;

/// Low-level RPC surface with fine-grained retry controls.
pub struct Chain {
	client: Client,
	retry_on_error: Option<bool>,
	retry_on_none: Option<bool>,
}
impl Chain {
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
	/// - `Ok(None)` when the block does not exist
	/// - `Err(RpcError)` when the underlying RPC call fails.
	pub async fn block_hash(&self, block_height: Option<u32>) -> Result<Option<H256>, RpcError> {
		let retry = self.should_retry_on_error();
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::chain::get_block_hash(&self.client.rpc_client, block_height).await };
		with_retry_on_error_and_none(f, retry, retry_on_none).await
	}

	/// Grabs a block header by hash or height.
	///
	/// # Returns
	/// - `Ok(Some(AvailHeader))` when the header exists.
	/// - `Ok(None)` when the header is missing
	/// - `Err(Error)` when conversions or RPC calls fail.
	pub async fn block_header(&self, at: Option<impl Into<HashStringNumber>>) -> Result<Option<AvailHeader>, Error> {
		async fn inner(r: &Chain, at: Option<HashStringNumber>) -> Result<Option<AvailHeader>, Error> {
			let retry_on_error = r.should_retry_on_error();
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

	/// Retrieves the full legacy block
	///
	/// # Returns
	/// - `Ok(Some(LegacyBlock))` when the block exists.
	/// - `Ok(None)` when the block is missing
	/// - `Err(Error)` when RPC calls fail.
	pub async fn legacy_block(&self, at: Option<H256>) -> Result<Option<LegacyBlock>, RpcError> {
		let retry = self.should_retry_on_error();
		let retry_on_none = self.retry_on_none.unwrap_or(false);

		let f = || async move { rpc::chain::get_block(&self.client.rpc_client, at).await };
		with_retry_on_error_and_none(f, retry, retry_on_none).await
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
	/// Returns `Err(Error)` when the account id cannot be parsed or the RPC call fails.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		async fn inner(r: &Chain, account_id: AccountIdLike) -> Result<u32, Error> {
			let retry_on_error = r.should_retry_on_error();
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
	/// Errors mirror [`Chain::account_info`].
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
		async fn inner(r: &Chain, account_id: AccountIdLike, at: HashStringNumber) -> Result<AccountInfo, Error> {
			let retry_on_error = r.should_retry_on_error();

			let account_id: AccountId = account_id.try_into().map_err(UserError::Other)?;
			let block_id: HashNumber = at.try_into().map_err(UserError::Other)?;
			let at = match block_id {
				HashNumber::Hash(h) => h,
				HashNumber::Number(n) => r
					.block_hash(Some(n))
					.await?
					.ok_or(UserError::Other("No block hash found for that block height".into()))?,
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
		async fn inner(r: &Chain, block_id: HashStringNumber) -> Result<BlockState, Error> {
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

			Ok(BlockState::Finalized)
		}

		inner(self, block_id.into()).await
	}

	/// Converts a block hash into its block height when possible.
	///
	/// # Returns
	/// - `Ok(Some(u32))` when the block height exists.
	/// - `Ok(None)` when the block height is missing
	/// - `Err(Error)` when RPC calls fail.
	pub async fn block_height(&self, at: impl Into<HashString>) -> Result<Option<u32>, Error> {
		async fn inner(r: &Chain, at: HashString) -> Result<Option<u32>, Error> {
			let retry_on_error = r.should_retry_on_error();
			let retry_on_none = r.retry_on_none.unwrap_or(false);

			let at: H256 = at.try_into().map_err(UserError::Other)?;
			let f = || async move { rpc::system::get_block_number(&r.client.rpc_client, at).await };
			Ok(with_retry_on_error_and_none(f, retry_on_error, retry_on_none).await?)
		}

		inner(self, at.into()).await
	}

	/// Returns the latest block info, either best or finalized.
	pub async fn block_info(&self, use_best_block: bool) -> Result<BlockInfo, RpcError> {
		let retry = self.should_retry_on_error();
		let f = || async move { rpc::system::latest_block_info(&self.client.rpc_client, use_best_block).await };
		with_retry_on_error(f, retry).await
	}

	/// Quick snapshot of both the best and finalized heads.
	pub async fn chain_info(&self) -> Result<ChainInfo, RpcError> {
		let retry = self.should_retry_on_error();

		let f = || async move { rpc::system::latest_chain_info(&self.client.rpc_client).await };
		with_retry_on_error(f, retry).await
	}

	/// Submits a signed extrinsic and gives you the transaction hash.
	///
	/// # Errors
	/// Returns `Err(RpcError)` when the node rejects the extrinsic or the RPC transport fails.
	pub async fn submit(&self, tx: &avail_rust_core::GenericExtrinsic<'_>) -> Result<H256, RpcError> {
		let retry = self.should_retry_on_error();
		let encoded = tx.encode();

		#[cfg(feature = "tracing")]
		if let Some(signed) = &tx.signature {
			if let avail_rust_core::MultiAddress::Id(account_id) = &signed.address {
				tracing::info!(target: "tx", "Submitting Transaction. Address: {}, Nonce: {}, App Id: {}", account_id, signed.tx_extra.nonce, signed.tx_extra.app_id);
			}
		}

		let enc_slice = encoded.as_slice();
		let f = || async move { rpc::author::submit_extrinsic(&self.client.rpc_client, enc_slice).await };
		let tx_hash = with_retry_on_error(f, retry).await?;

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
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(&self.client, &account_id, self.retry_on_error).await?;

		let tx_extra = avail_rust_core::ExtrinsicExtra::from(&refined_options);
		let tx_additional = avail_rust_core::ExtrinsicAdditional {
			spec_version: self.client.online_client().spec_version(),
			tx_version: self.client.online_client().transaction_version(),
			genesis_hash: self.client.online_client().genesis_hash(),
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
		let account_id = signer.public_key().to_account_id();
		let refined_options = options.build(&self.client, &account_id, self.retry_on_error).await?;

		let tx_extra = avail_rust_core::ExtrinsicExtra::from(&refined_options);
		let tx_additional = avail_rust_core::ExtrinsicAdditional {
			spec_version: self.client.online_client().spec_version(),
			tx_version: self.client.online_client().transaction_version(),
			genesis_hash: self.client.online_client().genesis_hash(),
			fork_hash: refined_options.mortality.block_hash,
		};

		let tx_payload = avail_rust_core::ExtrinsicPayload::new_borrowed(tx_call, tx_extra, tx_additional.clone());
		let tx_hash = self.sign_and_submit_payload(signer, tx_payload).await?;

		let value = SubmittedTransaction::new(self.client.clone(), tx_hash, account_id, refined_options, tx_additional);
		Ok(value)
	}

	/// Runs a `state_call` and returns the raw response string.
	pub async fn state_call(&self, method: &str, data: &[u8], at: Option<H256>) -> Result<String, RpcError> {
		let retry = self.should_retry_on_error();

		let f = || async move { rpc::state::call(&self.client.rpc_client, method, data, at).await };
		with_retry_on_error(f, retry).await
	}

	/// Downloads runtime metadata as bytes.
	pub async fn state_get_metadata(&self, at: Option<H256>) -> Result<Vec<u8>, RpcError> {
		let retry = self.should_retry_on_error();

		let f = || async move { rpc::state::get_metadata(&self.client.rpc_client, at).await };
		with_retry_on_error(f, retry).await
	}

	/// Reads a storage entry, returning the raw bytes if present.
	pub async fn state_get_storage(&self, key: &str, at: Option<H256>) -> Result<Option<Vec<u8>>, RpcError> {
		let retry = self.should_retry_on_error();

		let f = || async move { rpc::state::get_storage(&self.client.rpc_client, key, at).await };
		with_retry_on_error(f, retry).await
	}

	/// Lists storage keys under a prefix, one page at a time.
	pub async fn state_get_keys_paged(
		&self,
		prefix: Option<&str>,
		count: u32,
		start_key: Option<&str>,
		at: Option<H256>,
	) -> Result<Vec<String>, RpcError> {
		let retry = self.should_retry_on_error();

		let f =
			|| async move { rpc::state::get_keys_paged(&self.client.rpc_client, prefix, count, start_key, at).await };

		with_retry_on_error(f, retry).await
	}

	/// Performs a raw RPC invocation against the connected node and deserializes the response.
	pub async fn rpc_raw_call<T: serde::de::DeserializeOwned>(
		&self,
		method: &str,
		params: RpcParams,
	) -> Result<T, RpcError> {
		let retry = self.should_retry_on_error();

		let p = &params;
		let f = || async move { rpc::raw_call(&self.client.rpc_client, method, p.clone()).await };
		with_retry_on_error(f, retry).await
	}

	/// Calls into the runtime API and decodes the answer for you.
	pub async fn runtime_api_raw_call<T: codec::Decode>(
		&self,
		method: &str,
		data: &[u8],
		at: Option<H256>,
	) -> Result<T, RpcError> {
		let retry = self.should_retry_on_error();

		let f = || async move { runtime_api::raw_call(&self.client.rpc_client, method, data, at).await };
		with_retry_on_error(f, retry).await
	}

	/// Fetches GRANDPA justification for the given block number.
	///
	/// # Returns
	/// - `Ok(Some(GrandpaJustification))` when a justification is present.
	/// - `Ok(None)` when the runtime returns no justification.
	/// - `Err(RpcError)` if decoding the response or the RPC call fails.
	pub async fn grandpa_block_justification(&self, at: u32) -> Result<Option<GrandpaJustification>, RpcError> {
		let retry = self.should_retry_on_error();

		let f = || async move { rpc::grandpa::block_justification(&self.client.rpc_client, at).await };
		let result = with_retry_on_error(f, retry).await?;

		let Some(result) = result else {
			return Ok(None);
		};

		let justification = const_hex::decode(result.trim_start_matches("0x"))
			.map_err(|x| RpcError::MalformedResponse(x.to_string()))?;

		let justification = GrandpaJustification::decode(&mut justification.as_slice());
		let justification = justification.map_err(|e| RpcError::MalformedResponse(e.to_string()))?;
		Ok(Some(justification))
	}

	pub async fn transaction_payment_query_info(
		&self,
		extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, RpcError> {
		let retry = self.should_retry_on_error();

		let ext = &extrinsic;
		let f = || async move {
			runtime_api::api_transaction_payment_query_info(&self.client.rpc_client, ext.clone(), at).await
		};
		with_retry_on_error(f, retry).await
	}

	pub async fn transaction_payment_query_fee_details(
		&self,
		extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, RpcError> {
		let retry = self.should_retry_on_error();

		let ext = &extrinsic;
		let f = || async move {
			runtime_api::api_transaction_payment_query_fee_details(&self.client.rpc_client, ext.clone(), at).await
		};
		with_retry_on_error(f, retry).await
	}

	pub async fn transaction_payment_query_call_info(
		&self,
		call: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, RpcError> {
		let retry = self.should_retry_on_error();

		let c = &call;
		let f = || async move {
			runtime_api::api_transaction_payment_query_call_info(&self.client.rpc_client, c.clone(), at).await
		};
		with_retry_on_error(f, retry).await
	}

	pub async fn transaction_payment_query_call_fee_details(
		&self,
		call: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, RpcError> {
		let retry = self.should_retry_on_error();

		let c = &call;
		let f = || async move {
			runtime_api::api_transaction_payment_query_call_fee_details(&self.client.rpc_client, c.clone(), at).await
		};
		with_retry_on_error(f, retry).await
	}

	/// Retrieves the KATE block layout metadata (rows, cols, chunk size) for the block at `at`.
	///
	/// # Errors
	/// Returns `Err(RpcError)` when the KATE RPC call fails; respects the helper's retry policy.
	pub async fn kate_block_length(&self, at: Option<H256>) -> Result<BlockLength, RpcError> {
		let retry = self.should_retry_on_error();

		let f = || async move { rpc::kate::block_length(&self.client.rpc_client, at).await };
		with_retry_on_error(f, retry).await
	}

	/// Produces the KATE data proof (and optional addressed message) for the given extrinsic index.
	///
	/// # Errors
	/// Returns `Err(RpcError)` when the proof cannot be fetched or deserialised; obeys the retry setting.
	pub async fn kate_query_data_proof(
		&self,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, RpcError> {
		let retry = self.should_retry_on_error();

		let f = || async move { rpc::kate::query_data_proof(&self.client.rpc_client, transaction_index, at).await };
		with_retry_on_error(f, retry).await
	}

	/// Fetches individual KATE proofs for the provided list of cells.
	///
	/// # Errors
	/// Bubbles `Err(RpcError)` if the RPC call fails; retries follow the configured policy.
	pub async fn kate_query_proof(&self, cells: Vec<Cell>, at: Option<H256>) -> Result<Vec<GDataProof>, RpcError> {
		let retry = self.should_retry_on_error();

		let cells_ref = &cells;
		let f = || async move { rpc::kate::query_proof(&self.client.rpc_client, cells_ref.clone(), at).await };
		with_retry_on_error(f, retry).await
	}

	/// Returns KATE row data for the requested row indices (up to the chain-imposed limit).
	///
	/// # Errors
	/// Propagates `Err(RpcError)` when the row query fails; adheres to the retry preference.
	pub async fn kate_query_rows(&self, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, RpcError> {
		let retry = self.should_retry_on_error();

		let rows_ref = &rows;
		let f = || async move { rpc::kate::query_rows(&self.client.rpc_client, rows_ref.clone(), at).await };
		with_retry_on_error(f, retry).await
	}

	/// Requests multi-proofs for the supplied KATE cells, paired with the corresponding cell block metadata.
	///
	/// # Errors
	/// Returns `Err(RpcError)` when the RPC transport or decoding fails; follows the retry configuration.
	pub async fn kate_query_multi_proof(
		&self,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<(GMultiProof, GCellBlock)>, RpcError> {
		let retry = self.should_retry_on_error();

		let cells_ref = &cells;
		let f = || async move { rpc::kate::query_multi_proof(&self.client.rpc_client, cells_ref.clone(), at).await };
		with_retry_on_error(f, retry).await
	}

	#[cfg(feature = "next")]
	pub async fn blob_submit_blob(&self, metadata_signed_transaction: &[u8], blob: &[u8]) -> Result<(), Error> {
		let retry_on_error = self
			.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled());

		let f =
			|| async move { rpc::blob::submit_blob(&self.client.rpc_client, metadata_signed_transaction, blob).await };

		Ok(with_retry_on_error(f, retry_on_error).await?)
	}

	/// Fetches extrinsics from a block using the provided filters.
	///
	/// # Errors
	/// Returns `Err(Error)` when the block id cannot be decoded or the RPC request fails.
	pub async fn system_fetch_extrinsics(
		&self,
		block_id: impl Into<HashStringNumber>,
		opts: rpc::ExtrinsicOpts,
	) -> Result<Vec<ExtrinsicInfo>, Error> {
		async fn inner(
			r: &Chain,
			block_id: HashStringNumber,
			opts: &rpc::ExtrinsicOpts,
		) -> Result<Vec<ExtrinsicInfo>, Error> {
			let retry = r.should_retry_on_error();

			let block_id: HashNumber = block_id.try_into().map_err(UserError::Decoding)?;
			let f = || async move { rpc::system::fetch_extrinsics_v1(&r.client.rpc_client, block_id, opts).await };
			with_retry_on_error(f, retry).await.map_err(|e| e.into())
		}

		inner(self, block_id.into(), &opts).await
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
			r: &Chain,
			block_id: HashStringNumber,
			opts: &rpc::EventOpts,
		) -> Result<Vec<BlockPhaseEvent>, Error> {
			let retry = r.should_retry_on_error();

			let block_id: HashNumber = block_id.try_into().map_err(UserError::Decoding)?;
			let at = match block_id {
				HashNumber::Hash(x) => x,
				HashNumber::Number(x) => {
					let hash = r.block_hash(Some(x)).await?;
					hash.ok_or(RpcError::ExpectedData("Failed to fetch block height".into()))?
				},
			};

			let f = || async move { rpc::system::fetch_events_v1(&r.client.rpc_client, at, opts).await };
			with_retry_on_error(f, retry).await.map_err(|e| e.into())
		}

		inner(self, at.into(), &opts).await
	}

	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
	}
}
