use crate::{
	Client, Error, RetryPolicy, avail, conversions, error_ops, submission::SubmittedTransaction,
	subxt_signer::sr25519::Keypair, transaction_options::Options,
};
use avail::{
	balances::types::AccountData,
	system::{storage as SystemStorage, types::AccountInfo},
};
use avail_rust_core::{
	AccountId, AccountIdLike, AvailHeader, BlockInfo, Extension, ExtensionImplicit, H256, HashNumber, consensus,
	decoded_events::{EncodedEvent, parse_encoded_events},
	ext::subxt_rpcs::client::RpcParams,
	grandpa::GrandpaJustification,
	header::DigestItem,
	rpc::{
		self, Error as RpcError, LegacyBlock,
		blob::{Blob, BlobInfo},
		kate::{BlockLength, Cell, DataProof, GCellBlock, GDataProof, GMultiProof, GRow, ProofResponse},
		runtime_api,
	},
	substrate::{SignedPayload, StorageMap, StorageValue},
	types::{
		HashString,
		metadata::{ChainInfo, HashStringNumber},
		substrate::{FeeDetails, PerDispatchClassWeight, RuntimeDispatchInfo},
	},
};
use codec::{Decode, Encode};

/// Low-level chain RPC API with explicit retry controls.
pub struct Chain {
	pub(crate) client: Client,
	retry_on_error: RetryPolicy,
	retry_on_none: RetryPolicy,
}
impl Chain {
	/// Creates a chain helper bound to a client.
	pub fn new(client: Client) -> Self {
		Self {
			client,
			retry_on_error: RetryPolicy::Inherit,
			retry_on_none: RetryPolicy::Inherit,
		}
	}

	/// Sets retry behavior for RPC errors and `None` responses.
	pub fn retry_policy(mut self, error: RetryPolicy, none: RetryPolicy) -> Self {
		self.retry_on_error = error;
		self.retry_on_none = none;
		self
	}

	/// Returns block hash for a given height, or current best hash when `None`.
	pub async fn block_hash(&self, block_height: Option<u32>) -> Result<Option<H256>, RpcError> {
		let retry = self.should_retry_on_error();
		let retry_on_none = self.retry_on_none.resolve(false);

		retry_or_none!(retry, retry_on_none, {
			rpc::chain::get_block_hash(&self.client.rpc_client, block_height).await
		})
	}

	/// Returns a block header by hash/height, or best header when `None`.
	pub async fn block_header(&self, at: Option<impl Into<HashStringNumber>>) -> Result<Option<AvailHeader>, Error> {
		let retry_on_error = self.should_retry_on_error();
		let retry_on_none = self.retry_on_none.resolve(false);

		let at = if let Some(at) = at {
			Some(conversions::hash_string_number::to_hash(self, at).await?)
		} else {
			None
		};

		Ok(retry_or_none!(retry_on_error, retry_on_none, {
			rpc::chain::get_header(&self.client.rpc_client, at).await
		})?)
	}

	/// Returns a legacy block by hash, or best block when `None`.
	pub async fn legacy_block(&self, at: Option<H256>) -> Result<Option<LegacyBlock>, RpcError> {
		let retry = self.should_retry_on_error();
		let retry_on_none = self.retry_on_none.resolve(false);

		retry_or_none!(retry, retry_on_none, { rpc::chain::get_block(&self.client.rpc_client, at).await })
	}

	/// Fetches and decodes legacy block events.
	pub async fn legacy_block_events(&self, at: H256) -> Result<Vec<EncodedEvent>, Error> {
		let metadata = self.block_metadata(Some(at)).await?;
		let bytes = retry!(self.should_retry_on_error(), {
			avail::system::storage::Events::fetch(&self.client.rpc_client, Some(at)).await
		})?
		.ok_or_else(|| {
			Error::not_found_with_op(
				error_ops::ErrorOperation::ChainLegacyBlockEvents,
				"No events found for requested block",
			)
		})?;

		parse_encoded_events(&metadata, &bytes.0).ok_or_else(|| {
			Error::decode_with_op(
				error_ops::ErrorOperation::ChainLegacyBlockEvents,
				"Failed to decode legacy block events",
			)
		})
	}

	/// Fetches metadata at a specific block hash.
	pub async fn block_metadata(
		&self,
		at: Option<impl Into<HashStringNumber>>,
	) -> Result<avail_rust_core::ext::subxt_metadata::Metadata, Error> {
		let at = if let Some(at) = at {
			Some(conversions::hash_string_number::to_hash(self, at).await?)
		} else {
			None
		};

		retry!(self.should_retry_on_error(), {
			rpc::state::get_metadata(&self.client.rpc_client, at)
				.await
				.map_err(|e| e.into())
		})
	}

	/// Returns account nonce at a specific block.
	pub async fn block_nonce(
		&self,
		account_id: impl Into<AccountIdLike>,
		at: impl Into<HashStringNumber>,
	) -> Result<u32, Error> {
		self.account_info(account_id, at).await.map(|x| x.nonce)
	}

	/// Returns the latest account nonce.
	pub async fn account_nonce(&self, account_id: impl Into<AccountIdLike>) -> Result<u32, Error> {
		let account_id = conversions::account_id_like::to_account_id(account_id)?;
		let retry_on_error = self.should_retry_on_error();

		Ok(retry!(retry_on_error, {
			rpc::system::account_next_index(&self.client.rpc_client, &std::format!("{}", account_id)).await
		})?)
	}

	/// Reports the free balance for an account at a specific block.
	///
	/// Returns [`AccountData`] for the requested account at the chosen block.
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
	/// Returns [`AccountInfo`] containing balances, consumers, and nonce data.
	///
	/// missing, or the RPC call fails.
	pub async fn account_info(
		&self,
		account_id: impl Into<AccountIdLike>,
		at: impl Into<HashStringNumber>,
	) -> Result<AccountInfo, Error> {
		let account_id = conversions::account_id_like::to_account_id(account_id)?;
		let at = conversions::hash_string_number::to_hash(self, at).await?;
		let retry_on_error = self.should_retry_on_error();

		Ok(retry!(retry_on_error, {
			SystemStorage::Account::fetch(&self.client.rpc_client, &account_id, Some(at))
				.await
				.map(|x| x.unwrap_or_default())
		})?)
	}

	/// Converts a block hash into its block height when possible.
	///
	pub async fn block_height(&self, at: impl Into<HashString>) -> Result<Option<u32>, Error> {
		let at = conversions::hash_string::to_hash(at)?;
		let retry_on_error = self.should_retry_on_error();
		let retry_on_none = self.retry_on_none.resolve(false);

		Ok(retry_or_none!(retry_on_error, retry_on_none, {
			rpc::custom::get_block_number(&self.client.rpc_client, at).await
		})?)
	}

	/// Fetches block metadata for the provided block identifier.
	///
	/// Returns `BlockInfo` describing the block, or an error if the lookup fails.
	pub async fn block_info_from(&self, at: impl Into<HashStringNumber>) -> Result<BlockInfo, Error> {
		async fn inner(c: &Chain, at: HashNumber) -> Result<BlockInfo, Error> {
			let (height, hash) = match at {
				HashNumber::Hash(hash) => {
					let height = c.block_height(hash).await?;
					let Some(height) = height else {
						return Err(Error::not_found_with_op(
							error_ops::ErrorOperation::ChainBlockInfoFrom,
							std::format!("No block height found for hash: {}", hash),
						));
					};
					(height, hash)
				},
				HashNumber::Number(height) => {
					let hash = c.block_hash(Some(height)).await?;
					let Some(hash) = hash else {
						return Err(Error::not_found_with_op(
							error_ops::ErrorOperation::ChainBlockInfoFrom,
							std::format!("No block hash found for height: {}", height),
						));
					};
					(height, hash)
				},
				HashNumber::HashAndNumber((h, n)) => (n, h),
			};

			Ok(BlockInfo::from((hash, height)))
		}

		let at = HashNumber::try_from(at.into())
			.map_err(|e| Error::validation_with_op(error_ops::ErrorOperation::ChainBlockInfoFrom, e.to_string()))?;
		inner(self, at).await
	}

	/// Determines the author of the specified block.
	///
	/// Returns the account id of the block author or an error if it cannot be determined.
	pub async fn block_author(&self, at: impl Into<HashStringNumber>) -> Result<AccountId, Error> {
		let hash = conversions::hash_string_number::to_hash(self, at).await?;

		let header = self.block_header(Some(hash)).await?;
		let Some(header) = header else {
			return Err(Error::not_found_with_op(
				error_ops::ErrorOperation::ChainBlockAuthor,
				"No block header found for requested block",
			));
		};

		for item in &header.digest.logs {
			let (id, value) = match &item {
				DigestItem::PreRuntime(id, value) => (id, value),
				_ => continue,
			};

			if !id.eq(&consensus::babe::BABE_ENGINE_ID) {
				continue;
			}

			let mut v = value.as_slice();
			let pre_digest = consensus::babe::PreDigest::decode(&mut v).map_err(|e| Error::Decode(e.to_string()))?;

			let validators = avail::session::storage::Validators::fetch(&self.client.rpc_client, Some(hash)).await?;
			let Some(validators) = validators else {
				return Err(Error::not_found_with_op(
					error_ops::ErrorOperation::ChainBlockAuthor,
					std::format!("No validators found for block hash: {:?}", hash),
				));
			};

			if let Some(account_id) = validators.get(pre_digest.authority_index() as usize) {
				return Ok(account_id.clone());
			}
		}

		Err(Error::not_found_with_op(
			error_ops::ErrorOperation::ChainBlockAuthor,
			std::format!("Failed to find block author for block hash: {}", hash),
		))
	}

	/// Counts the events emitted by the specified block.
	///
	/// Returns the number of events as `usize`, or an error if the count cannot be fetched.
	pub async fn block_event_count(&self, at: impl Into<HashStringNumber>) -> Result<usize, Error> {
		let hash = conversions::hash_string_number::to_hash(self, at).await?;
		let retry_on_error = self.should_retry_on_error();

		let count = retry_or_none!(retry_on_error, false, {
			avail::system::storage::EventCount::fetch(&self.client.rpc_client, Some(hash)).await
		})?;
		let Some(count) = count else {
			return Err(Error::not_found_with_op(
				error_ops::ErrorOperation::ChainBlockEventCount,
				std::format!("No block event count found at block hash: {:?}", hash),
			));
		};

		Ok(count as usize)
	}

	/// Retrieves the dispatch-class weight totals for the specified block.
	///
	/// Returns the per-dispatch-class weight totals or an error if unavailable.
	pub async fn block_weight(&self, at: impl Into<HashStringNumber>) -> Result<PerDispatchClassWeight, Error> {
		let hash = conversions::hash_string_number::to_hash(self, at).await?;
		let retry_on_error = self.should_retry_on_error();

		let weight = retry_or_none!(retry_on_error, false, {
			avail::system::storage::BlockWeight::fetch(&self.client.rpc_client, Some(hash)).await
		})?;
		let Some(weight) = weight else {
			return Err(Error::not_found_with_op(
				error_ops::ErrorOperation::ChainBlockWeight,
				std::format!("No block weight found at block hash: {:?}", hash),
			));
		};

		Ok(weight)
	}

	/// Quick snapshot of both the best and finalized heads.
	pub async fn info(&self) -> Result<ChainInfo, RpcError> {
		retry!(self.should_retry_on_error(), { rpc::custom::chain_info(&self.client.rpc_client).await })
	}

	/// Builds a payload from a call and signs it with sensible defaults.
	///
	pub async fn build_extrinsic_from_call<'a>(
		&self,
		signer: &Keypair,
		call: &'a [u8],
		options: Options,
	) -> Result<avail_rust_core::ExtrinsicBorrowed<'a>, Error> {
		let account_id = signer.public_key().to_account_id();

		let resolved = options.resolve(&self.client, &account_id, self.retry_on_error).await?;

		let extension = avail_rust_core::Extension::from(&resolved);
		let implicit = avail_rust_core::ExtensionImplicit {
			spec_version: self.client.online_client().spec_version(),
			tx_version: self.client.online_client().transaction_version(),
			genesis_hash: self.client.online_client().genesis_hash(),
			fork_hash: resolved.mortality.block_hash,
		};

		let signature = avail_rust_core::SignedPayload::sign_static(call, &extension, &implicit, signer);

		Ok(avail_rust_core::ExtrinsicBorrowed::new_signed(account_id, signature, extension, call))
	}

	pub async fn submit(&self, extrinsic: &[u8]) -> Result<H256, RpcError> {
		retry!(self.should_retry_on_error(), {
			rpc::author::submit_extrinsic(&self.client.rpc_client, extrinsic).await
		})
	}

	/// Signs the payload and submits it in one step.
	pub async fn sign_and_submit_payload(
		&self,
		signer: &Keypair,
		payload: SignedPayload<'_>,
	) -> Result<H256, RpcError> {
		let signature = payload.sign(signer);

		let account_id = signer.public_key().to_account_id();
		let tx = avail_rust_core::ExtrinsicBorrowed::new_signed(
			account_id,
			signature,
			payload.extension.clone(),
			payload.call,
		);
		let tx_hash = self.submit(&tx.encode()).await?;

		Ok(tx_hash)
	}

	/// Signs a call, submits it, and hands back a tracker you can poll.
	///
	///   receipt queries.
	pub async fn sign_and_submit_call(
		&self,
		signer: &Keypair,
		call: &[u8],
		options: Options,
	) -> Result<SubmittedTransaction, Error> {
		let account_id = signer.public_key().to_account_id();
		let resolved = options.resolve(&self.client, &account_id, self.retry_on_error).await?;

		let extension = Extension::from(&resolved);
		let implicit = ExtensionImplicit {
			spec_version: self.client.online_client().spec_version(),
			tx_version: self.client.online_client().transaction_version(),
			genesis_hash: self.client.online_client().genesis_hash(),
			fork_hash: resolved.mortality.block_hash,
		};

		let tx_payload = SignedPayload::new(call, &extension, &implicit);
		let ext_hash = self.sign_and_submit_payload(signer, tx_payload).await?;

		let start = resolved.mortality.block_height;
		let end = resolved.mortality.period as u32 + start;

		Ok(SubmittedTransaction::new(self.client.clone(), ext_hash, start, end))
	}

	/// Runs a `state_call` and returns the raw response string.
	pub async fn state_call(&self, method: &str, data: &[u8], at: Option<H256>) -> Result<String, RpcError> {
		retry!(self.should_retry_on_error(), { rpc::state::call(&self.client.rpc_client, method, data, at).await })
	}

	/// Downloads runtime metadata as bytes.
	pub async fn state_get_metadata(&self, at: Option<H256>) -> Result<Vec<u8>, RpcError> {
		retry!(self.should_retry_on_error(), { rpc::state::get_metadata_bytes(&self.client.rpc_client, at).await })
	}

	/// Reads a storage entry, returning the raw bytes if present.
	pub async fn state_get_storage(&self, key: &str, at: Option<H256>) -> Result<Option<Vec<u8>>, RpcError> {
		retry!(self.should_retry_on_error(), { rpc::state::get_storage(&self.client.rpc_client, key, at).await })
	}

	/// Lists storage keys under a prefix, one page at a time.
	pub async fn state_get_keys_paged(
		&self,
		prefix: Option<&str>,
		count: u32,
		start_key: Option<&str>,
		at: Option<H256>,
	) -> Result<Vec<String>, RpcError> {
		retry!(self.should_retry_on_error(), {
			rpc::state::get_keys_paged(&self.client.rpc_client, prefix, count, start_key, at).await
		})
	}

	/// Performs a raw RPC invocation against the connected node and deserializes the response.
	pub async fn rpc_raw_call<T: serde::de::DeserializeOwned>(
		&self,
		method: &str,
		params: RpcParams,
	) -> Result<T, RpcError> {
		retry!(self.should_retry_on_error(), {
			rpc::raw_call(&self.client.rpc_client, method, params.clone()).await
		})
	}

	/// Calls into the runtime API and decodes the answer for you.
	pub async fn runtime_api_raw_call<T: codec::Decode>(
		&self,
		method: &str,
		data: &[u8],
		at: Option<H256>,
	) -> Result<T, RpcError> {
		retry!(self.should_retry_on_error(), {
			runtime_api::raw_call(&self.client.rpc_client, method, data, at).await
		})
	}

	/// Fetches GRANDPA justification for the given block number.
	///
	pub async fn block_justification(
		&self,
		at: impl Into<HashStringNumber>,
	) -> Result<Option<GrandpaJustification>, Error> {
		async fn inner(c: &Chain, at: HashNumber) -> Result<Option<GrandpaJustification>, Error> {
			let at = match at {
				HashNumber::Hash(h) => c.block_height(h).await?,
				HashNumber::Number(n) => Some(n),
				HashNumber::HashAndNumber((_, n)) => Some(n),
			};
			let Some(at) = at else {
				return Err(Error::not_found_with_op(
					error_ops::ErrorOperation::ChainBlockJustification,
					"No block found for requested hash",
				));
			};

			let result = retry!(c.should_retry_on_error(), {
				rpc::grandpa::block_justification(&c.client.rpc_client, at).await
			})?;

			let Some(result) = result else {
				return Ok(None);
			};

			let justification = const_hex::decode(result.trim_start_matches("0x"))
				.map_err(|x| RpcError::MalformedResponse(x.to_string()))?;

			let justification = GrandpaJustification::decode(&mut justification.as_slice());
			let justification = justification.map_err(|e| RpcError::MalformedResponse(e.to_string()))?;
			Ok(Some(justification))
		}

		let at = HashNumber::try_from(at.into())
			.map_err(|e| Error::validation_with_op(error_ops::ErrorOperation::ChainBlockJustification, e))?;
		inner(self, at).await
	}

	/// Queries the runtime for fee information about an encoded extrinsic.
	///
	/// Returns dispatch info describing the estimated fee and weight.
	pub async fn transaction_payment_query_info(
		&self,
		extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, RpcError> {
		let extrinsic = &extrinsic;
		retry!(self.should_retry_on_error(), {
			runtime_api::api_transaction_payment_query_info(&self.client.rpc_client, extrinsic.clone(), at).await
		})
	}

	/// Retrieves detailed fee breakdown for an encoded extrinsic.
	///
	/// Returns fee components such as inclusion and tip fees.
	pub async fn transaction_payment_query_fee_details(
		&self,
		extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, RpcError> {
		let extrinsic = &extrinsic;
		retry!(self.should_retry_on_error(), {
			runtime_api::api_transaction_payment_query_fee_details(&self.client.rpc_client, extrinsic.clone(), at).await
		})
	}

	/// Queries the runtime for fee information about an encoded call.
	///
	/// Returns dispatch info describing the estimated fee and weight.
	pub async fn transaction_payment_query_call_info(
		&self,
		call: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, RpcError> {
		let call = &call;
		retry!(self.should_retry_on_error(), {
			runtime_api::api_transaction_payment_query_call_info(&self.client.rpc_client, call.clone(), at).await
		})
	}

	/// Retrieves detailed fee components for an encoded call.
	///
	/// Returns the fee breakdown for executing the call.
	pub async fn transaction_payment_query_call_fee_details(
		&self,
		call: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, RpcError> {
		let call = &call;
		retry!(self.should_retry_on_error(), {
			runtime_api::api_transaction_payment_query_call_fee_details(&self.client.rpc_client, call.clone(), at).await
		})
	}

	/// Retrieves the KATE block layout metadata (rows, cols, chunk size) for the block at `at`.
	///
	pub async fn kate_block_length(&self, at: Option<H256>) -> Result<BlockLength, RpcError> {
		retry!(self.should_retry_on_error(), { rpc::kate::block_length(&self.client.rpc_client, at).await })
	}

	/// Produces the KATE data proof (and optional addressed message) for the given extrinsic index.
	///
	pub async fn kate_query_data_proof(
		&self,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, RpcError> {
		retry!(self.should_retry_on_error(), {
			rpc::kate::query_data_proof(&self.client.rpc_client, transaction_index, at).await
		})
	}

	/// Fetches individual KATE proofs for the provided list of cells.
	///
	/// Bubbles `Err(RpcError)` if the RPC call fails; retries follow the configured policy.
	pub async fn kate_query_proof(&self, cells: Vec<Cell>, at: Option<H256>) -> Result<Vec<GDataProof>, RpcError> {
		let cells = &cells;
		retry!(self.should_retry_on_error(), {
			rpc::kate::query_proof(&self.client.rpc_client, cells.clone(), at).await
		})
	}

	/// Returns KATE row data for the requested row indices (up to the chain-imposed limit).
	///
	/// Propagates `Err(RpcError)` when the row query fails; adheres to the retry preference.
	pub async fn kate_query_rows(&self, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, RpcError> {
		let rows = &rows;
		retry!(self.should_retry_on_error(), {
			rpc::kate::query_rows(&self.client.rpc_client, rows.clone(), at).await
		})
	}

	/// Requests multi-proofs for the supplied KATE cells, paired with the corresponding cell block metadata.
	///
	pub async fn kate_query_multi_proof(
		&self,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<(GMultiProof, GCellBlock)>, RpcError> {
		let cells = &cells;
		retry!(self.should_retry_on_error(), {
			rpc::kate::query_multi_proof(&self.client.rpc_client, cells.clone(), at).await
		})
	}

	/// Submits a blob alongside its signed metadata transaction.
	///
	/// Returns `Ok(())` on success or an error if the submission fails.
	pub async fn blob_submit_blob(&self, metadata_signed_transaction: &[u8], blob: &[u8]) -> Result<(), Error> {
		Ok(retry!(self.should_retry_on_error(), {
			rpc::blob::submit_blob(&self.client.rpc_client, metadata_signed_transaction, blob).await
		})?)
	}

	pub async fn blob_get_blob(&self, blob_hash: H256, block_hash: Option<H256>) -> Result<Blob, Error> {
		Ok(retry!(self.should_retry_on_error(), {
			rpc::blob::get_blob(&self.client.rpc_client, blob_hash, block_hash).await
		})?)
	}

	/// Retrieve indexed blob info
	pub async fn blob_get_blob_info(&self, blob_hash: H256) -> Result<BlobInfo, Error> {
		Ok(retry!(self.should_retry_on_error(), {
			rpc::blob::get_blob_info(&self.client.rpc_client, blob_hash).await
		})?)
	}

	/// Return inclusion proof for a blob. If `at` is `Some(hash)` the proof is computed for that block,
	/// otherwise the node will try to use its indexed finalized block for the blob.
	pub async fn blob_inclusion_proof(&self, blob_hash: H256, at: Option<H256>) -> Result<DataProof, Error> {
		Ok(retry!(self.should_retry_on_error(), {
			rpc::blob::inclusion_proof(&self.client.rpc_client, blob_hash, at).await
		})?)
	}

	/// Fetches extrinsics from a block using the provided filters.
	///
	pub async fn extrinsics(
		&self,
		at: impl Into<HashStringNumber>,
		allow_list: Option<Vec<rpc::AllowedExtrinsic>>,
		sig_filter: rpc::SignatureFilter,
		data_format: rpc::DataFormat,
	) -> Result<Vec<rpc::Extrinsic>, Error> {
		async fn inner(
			c: &Chain,
			at: HashNumber,
			allow_list: Option<Vec<rpc::AllowedExtrinsic>>,
			sig_filter: rpc::SignatureFilter,
			data_format: rpc::DataFormat,
		) -> Result<Vec<rpc::Extrinsic>, Error> {
			retry!(c.should_retry_on_error(), {
				rpc::custom::fetch_extrinsics(
					&c.client.rpc_client,
					at.into(),
					allow_list.clone(),
					sig_filter.clone(),
					data_format,
				)
				.await
				.map_err(|e| e.into())
			})
		}

		let at = HashNumber::try_from(at.into())
			.map_err(|e| Error::validation_with_op(error_ops::ErrorOperation::ChainFetchExtrinsics, e))?;
		inner(self, at, allow_list, sig_filter, data_format).await
	}

	/// Pulls events for a block with optional filtering.
	///
	pub async fn events(
		&self,
		at: impl Into<HashStringNumber>,
		allow_list: rpc::AllowedEvents,
		fetch_data: bool,
	) -> Result<Vec<rpc::PhaseEvents>, Error> {
		async fn inner(
			c: &Chain,
			at: HashNumber,
			allow_list: rpc::AllowedEvents,
			fetch_data: bool,
		) -> Result<Vec<rpc::PhaseEvents>, Error> {
			retry!(c.should_retry_on_error(), {
				rpc::custom::fetch_events(&c.client.rpc_client, at.into(), allow_list.clone(), fetch_data)
					.await
					.map_err(|e| e.into())
			})
		}

		let at = HashNumber::try_from(at.into())
			.map_err(|e| Error::validation_with_op(error_ops::ErrorOperation::ChainFetchEvents, e.to_string()))?;
		inner(self, at, allow_list, fetch_data).await
	}

	pub async fn block_timestamp(&self, at: impl Into<HashStringNumber>) -> Result<u64, Error> {
		async fn inner(c: &Chain, at: HashNumber) -> Result<u64, Error> {
			retry!(c.should_retry_on_error(), {
				rpc::custom::block_timestamp(&c.client.rpc_client, at.into())
					.await
					.map_err(|e| e.into())
			})
		}

		let at = HashNumber::try_from(at.into())
			.map_err(|e| Error::validation_with_op(error_ops::ErrorOperation::ChainBlockTimestamp, e.to_string()))?;
		inner(self, at).await
	}

	/// Reports whether RPC helpers should retry after encountering errors.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.resolve(self.client.retry_policy() != RetryPolicy::Disabled)
	}
}
