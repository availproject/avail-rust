//! Builders for submitting extrinsics and inspecting their on-chain lifecycle.

use crate::{
	Client, Error, UserError,
	block_api::{
		BlockApi, BlockEvents, BlockExtrinsic, BlockRawExtrinsic, BlockTransaction, BlockWithExt, BlockWithRawExt,
		BlockWithTx, ExtrinsicEvents,
	},
	subscription::Sub,
	subxt_signer::sr25519::Keypair,
	transaction_options::{Options, RefinedMortality, RefinedOptions},
};
use avail_rust_core::{
	AccountId, BlockInfo, EncodeSelector, H256, HasHeader, RpcError,
	ext::codec::Encode,
	substrate::extrinsic::{ExtrinsicAdditional, ExtrinsicCall, GenericExtrinsic},
	types::{
		metadata::{HashString, TransactionRef},
		substrate::{FeeDetails, RuntimeDispatchInfo},
	},
};
use codec::Decode;
#[cfg(feature = "tracing")]
use tracing::info;

/// Builder that keeps an encoded call together with the client connection and exposes helpers for
/// signing, submitting, and querying execution costs.
#[derive(Clone)]
pub struct SubmittableTransaction {
	client: Client,
	pub call: ExtrinsicCall,
	retry_on_error: Option<bool>,
}

impl SubmittableTransaction {
	/// Creates a transaction builder from an encoded call.
	///
	/// The builder is inert until one of the async helpers is invoked. By default it inherits the
	/// client's retry policy, but this can be customised via [`set_retry_on_error`](Self::set_retry_on_error).
	pub fn new(client: Client, call: ExtrinsicCall) -> Self {
		Self { client, call, retry_on_error: None }
	}

	/// Signs the call with the provided keypair and submits it to the chain in a single RPC round-trip.
	///
	/// # Returns
	/// - `Ok(SubmittedTransaction)` when the node accepts the extrinsic and returns its hash along with
	///   metadata inferred from `options`.
	/// - `Err(Error)` when signing fails, submission is rejected by the node, or any underlying RPC call
	///   (potentially retried according to the configured policy) returns an error.
	///
	/// The submission uses `options` (nonce, tip, mortality) exactly as provided; no additional mutation
	/// happens inside this helper.
	pub async fn sign_and_submit(&self, signer: &Keypair, options: Options) -> Result<SubmittedTransaction, Error> {
		self.client
			.chain()
			.retry_on(self.retry_on_error, None)
			.sign_and_submit_call(signer, &self.call, options)
			.await
	}

	/// Signs the call without submitting it, returning the encoded extrinsic bytes that would be sent
	/// to the network.
	///
	/// # Returns
	/// - `Ok(GenericExtrinsic<'_>)` containing the SCALE-encoded payload ready for submission.
	/// - `Err(Error)` when the signing operation fails (for example, due to a bad signer, stale
	///   account information, or RPC issues while fetching metadata).
	pub async fn sign(&self, signer: &Keypair, options: Options) -> Result<GenericExtrinsic<'_>, Error> {
		self.client
			.chain()
			.retry_on(self.retry_on_error, None)
			.sign_call(signer, &self.call, options)
			.await
	}

	/// Estimates fee details for the underlying call using runtime information at `at` without signing
	/// or submitting anything.
	///
	/// # Returns
	/// - `Ok(FeeDetails)` containing the partial fee breakdown the runtime reports for the call.
	/// - `Err(RpcError)` if the node rejects the dry-run query (e.g. bad call data, missing runtime
	///   exposes) or if transport errors occur.
	pub async fn estimate_call_fees(&self, at: Option<H256>) -> Result<FeeDetails, RpcError> {
		let call = self.call.encode();
		self.client
			.chain()
			.retry_on(self.retry_on_error, None)
			.transaction_payment_query_call_fee_details(call, at)
			.await
	}

	/// Signs the call with the provided options and queries the chain for the cost of submitting that
	/// exact extrinsic.
	///
	/// # Returns
	/// - `Ok(FeeDetails)` containing the fee components returned by the runtime.
	/// - `Err(Error)` if signing the call fails or if the fee query returns an error (in which case the
	///   underlying [`RpcError`] is wrapped in the returned [`Error`]).
	pub async fn estimate_extrinsic_fees(
		&self,
		signer: &Keypair,
		options: Options,
		at: Option<H256>,
	) -> Result<FeeDetails, Error> {
		let transaction = self.sign(signer, options).await?;
		let transaction = transaction.encode();
		Ok(self
			.client
			.chain()
			.retry_on(self.retry_on_error, None)
			.transaction_payment_query_fee_details(transaction, at)
			.await?)
	}

	/// Returns runtime dispatch information for the call, including weight, class, and partial fee
	/// estimation based on the provided block context.
	///
	/// # Returns
	/// - `Ok(RuntimeDispatchInfo)` with weight and class metadata.
	/// - `Err(RpcError)` if the node cannot evaluate the call (bad parameters, runtime error, or RPC
	///   transport failure).
	pub async fn call_info(&self, at: Option<H256>) -> Result<RuntimeDispatchInfo, RpcError> {
		let call = self.call.encode();
		self.client
			.chain()
			.retry_on(self.retry_on_error, None)
			.transaction_payment_query_call_info(call, at)
			.await
	}

	/// Resolves whether RPC calls performed through this builder should be retried on transient
	/// failures.
	///
	/// The method returns the explicit override set by [`set_retry_on_error`](Self::set_retry_on_error),
	/// falling back to the client's global retry configuration when no override is present.
	pub fn should_retry_on_error(&self) -> bool {
		should_retry(&self.client, self.retry_on_error)
	}

	/// Controls retry behaviour for RPC calls sent via this builder.
	///
	/// # Parameters
	/// - `Some(true)`: force retries regardless of the client's global setting.
	/// - `Some(false)`: disable retries for requests issued through this builder.
	/// - `None`: fall back to the client's global retry configuration.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.retry_on_error = value;
	}

	/// Converts any encodable call into a `SubmittableTransaction` based on its pallet and call indices.
	/// The provided value is SCALE-encoded immediately; failures propagate as panics originating from
	/// the underlying encoding implementation.
	pub fn from_encodable<T: HasHeader + Encode>(client: Client, value: T) -> SubmittableTransaction {
		let call = ExtrinsicCall::new(T::HEADER_INDEX.0, T::HEADER_INDEX.1, value.encode());
		SubmittableTransaction::new(client, call)
	}

	/// Hashes the call payload as it would appear in an extrinsic, returning the blake2 hash used by
	/// the runtime for call identification.
	pub fn call_hash(&self) -> [u8; 32] {
		self.call.hash()
	}
}

impl Into<ExtrinsicCall> for SubmittableTransaction {
	fn into(self) -> ExtrinsicCall {
		self.call
	}
}

impl Into<ExtrinsicCall> for &SubmittableTransaction {
	fn into(self) -> ExtrinsicCall {
		self.call.clone()
	}
}

/// Handle to a transaction that has already been submitted to the network along with the contextual
/// information required to query its lifecycle.
#[derive(Clone)]
pub struct SubmittedTransaction {
	client: Client,
	pub tx_hash: H256,
	pub account_id: AccountId,
	pub options: RefinedOptions,
	pub additional: ExtrinsicAdditional,
}

impl SubmittedTransaction {
	/// Creates a new submitted transaction handle using previously gathered metadata.
	///
	/// This does not perform any network calls; it simply stores the information needed to later
	/// resolve receipts or query status.
	pub fn new(
		client: Client,
		tx_hash: H256,
		account_id: AccountId,
		options: RefinedOptions,
		additional: ExtrinsicAdditional,
	) -> Self {
		Self { client, tx_hash, account_id, options, additional }
	}

	/// Produces a receipt describing how the transaction landed on chain, if it did at all.
	///
	/// # Returns
	/// - `Ok(Some(TransactionReceipt))` when the transaction is found in the searched block range.
	/// - `Ok(None)` when the transaction cannot be located within the mortality window implied by
	///   `options`.
	/// - `Err(Error)` when the underlying RPC or subscription queries fail.
	///
	/// Set `use_best_block` to `true` to follow the node's best chain (potentially including
	/// non-finalized blocks) or `false` to restrict the search to finalized blocks.
	pub async fn receipt(&self, use_best_block: bool) -> Result<Option<TransactionReceipt>, Error> {
		Utils::transaction_receipt(
			self.client.clone(),
			self.tx_hash,
			self.options.nonce,
			&self.account_id,
			&self.options.mortality,
			use_best_block,
		)
		.await
	}
}

/// Indicates what happened to a transaction after it was submitted.
///
/// The variants correspond to the states returned by the chain RPC when querying transaction status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockState {
	/// The transaction was included in a block but the block may still be re-orged out.
	Included = 0,
	/// The block containing the transaction is finalized and immutable under normal circumstances.
	Finalized = 1,
	/// The transaction was seen but ended up discarded (e.g. due to invalidation).
	Discarded = 2,
	/// The transaction could not be found on chain.
	DoesNotExist = 3,
}

/// Detailed information about where a transaction was found on chain.
#[derive(Clone)]
pub struct TransactionReceipt {
	client: Client,
	pub block_ref: BlockInfo,
	pub tx_ref: TransactionRef,
}

impl TransactionReceipt {
	/// Wraps the provided block and transaction references without performing network IO.
	pub fn new(client: Client, block: BlockInfo, tx: TransactionRef) -> Self {
		Self { client, block_ref: block, tx_ref: tx }
	}

	/// Returns the current lifecycle state of the containing block.
	///
	/// # Returns
	/// - `Ok(BlockState)` on success.
	/// - `Err(Error)` if the RPC request fails or the node cannot provide the block state.
	pub async fn block_state(&self) -> Result<BlockState, Error> {
		self.client.chain().block_state(self.block_ref.hash).await
	}

	/// Fetches and decodes the transaction at the recorded index within the block.
	///
	/// # Returns
	/// - `Ok(BlockTransaction<T>)` when the transaction exists and can be decoded as `T`.
	/// - `Err(Error)` if the transaction is missing, cannot be decoded, or any RPC call fails.
	pub async fn tx<T: HasHeader + Decode>(&self) -> Result<BlockTransaction<T>, Error> {
		let block = BlockWithTx::new(self.client.clone(), self.block_ref.height);
		let tx = block.get(self.tx_ref.index).await?;
		let Some(tx) = tx else {
			return Err(RpcError::ExpectedData("No transaction found at the requested index.".into()).into());
		};

		Ok(tx)
	}

	/// Fetches and decodes the extrinsic at the recorded index within the block.
	///
	/// # Returns
	/// - `Ok(BlockExtrinsic<T>)` when the extrinsic exists and decodes as `T`.
	/// - `Err(Error)` when the extrinsic is missing, cannot be decoded as `T`, or RPC access fails.
	pub async fn ext<T: HasHeader + Decode>(&self) -> Result<BlockExtrinsic<T>, Error> {
		let block = BlockWithExt::new(self.client.clone(), self.block_ref.height);
		let ext: Option<BlockExtrinsic<T>> = block.get(self.tx_ref.index).await?;
		let Some(ext) = ext else {
			return Err(RpcError::ExpectedData("No extrinsic found at the requested index.".into()).into());
		};

		Ok(ext)
	}

	/// Fetches just the call payload for the extrinsic at the recorded index.
	///
	/// # Returns
	/// - `Ok(T)` when the extrinsic exists and its call decodes as `T`.
	/// - `Err(Error)` otherwise (missing extrinsic, decode failure, or RPC error).
	pub async fn call<T: HasHeader + Decode>(&self) -> Result<T, Error> {
		let block = BlockWithExt::new(self.client.clone(), self.block_ref.height);
		let tx = block.get(self.tx_ref.index).await?;
		let Some(tx) = tx else {
			return Err(RpcError::ExpectedData("No extrinsic found at the requested index.".into()).into());
		};

		Ok(tx.call)
	}

	/// Returns the raw extrinsic bytes or a different encoding if requested.
	///
	/// # Returns
	/// - `Ok(BlockRawExtrinsic)` with the requested encoding.
	/// - `Err(Error)` when the extrinsic cannot be found or an RPC failure occurs.
	pub async fn raw_ext(&self, encode_as: EncodeSelector) -> Result<BlockRawExtrinsic, Error> {
		let block = BlockWithRawExt::new(self.client.clone(), self.block_ref.height);
		let ext = block.get(self.tx_ref.index, encode_as).await?;
		let Some(ext) = ext else {
			return Err(RpcError::ExpectedData("No extrinsic found at the requested index.".into()).into());
		};

		Ok(ext)
	}

	/// Fetches the events emitted as part of the transaction execution.
	///
	/// # Returns
	/// - `Ok(ExtrinsicEvents)` when the extrinsic exists and events are available.
	/// - `Err(Error)` when the events cannot be located or fetched.
	pub async fn events(&self) -> Result<ExtrinsicEvents, Error> {
		let block = BlockEvents::new(self.client.clone(), self.block_ref.hash);
		let events = block.ext(self.tx_ref.index).await?;
		let Some(events) = events else {
			return Err(RpcError::ExpectedData("No events found for the requested extrinsic.".into()).into());
		};
		Ok(events)
	}

	/// Iterates block-by-block from `block_start` through `block_end` (inclusive) looking for an
	/// extrinsic whose hash matches `tx_hash`.
	///
	/// Returns `Ok(Some(TransactionReceipt))` as soon as a match is found, `Ok(None)` when the
	/// entire range has been exhausted without a match, and bubbles up any RPC or subscription
	/// errors encountered along the way.
	///
	/// Fails fast with a validation error when `block_start > block_end`. When `use_best_block`
	/// is `true`, the search follows the node's best chain; otherwise it restricts the iteration to
	/// finalized blocks only.
	pub async fn from_range(
		client: Client,
		tx_hash: impl Into<HashString>,
		block_start: u32,
		block_end: u32,
		use_best_block: bool,
	) -> Result<Option<TransactionReceipt>, Error> {
		if block_start > block_end {
			return Err(UserError::ValidationFailed("Block Start cannot start after Block End".into()).into());
		}
		let tx_hash: HashString = tx_hash.into();
		let mut sub = Sub::new(client.clone());
		sub.use_best_block(use_best_block);
		sub.set_block_height(block_start);

		loop {
			let block_ref = sub.next().await?;

			let block = BlockWithRawExt::new(client.clone(), block_ref.height);
			let ext = block.get(tx_hash.clone(), EncodeSelector::None).await?;
			if let Some(ext) = ext {
				let tr = TransactionReceipt::new(client.clone(), block_ref, (ext.ext_hash(), ext.ext_index()).into());
				return Ok(Some(tr));
			}

			if block_ref.height >= block_end {
				return Ok(None);
			}
		}
	}
}

/// Convenience helpers for locating transactions on chain.
pub struct Utils;
impl Utils {
	/// Resolves the canonical receipt for a transaction if it landed on chain within its mortality window.
	///
	/// # Returns
	/// - `Ok(Some(TransactionReceipt))` when a matching inclusion is located.
	/// - `Ok(None)` when no matching transaction exists in the searched range.
	/// - `Err(Error)` when RPC queries fail or input validation detects an inconsistency.
	pub async fn transaction_receipt(
		client: Client,
		tx_hash: H256,
		nonce: u32,
		account_id: &AccountId,
		mortality: &RefinedMortality,
		use_best_block: bool,
	) -> Result<Option<TransactionReceipt>, Error> {
		let Some(block_ref) =
			Self::find_correct_block_info(&client, nonce, tx_hash, account_id, mortality, use_best_block).await?
		else {
			return Ok(None);
		};

		let block = BlockApi::new(client.clone(), block_ref.hash);
		let ext = block.raw_ext().get(tx_hash, EncodeSelector::None).await?;

		let Some(ext) = ext else {
			return Ok(None);
		};

		let tx_ref = TransactionRef::from((ext.ext_hash(), ext.ext_index()));
		Ok(Some(TransactionReceipt::new(client, block_ref, tx_ref)))
	}

	/// Inspects blocks following the transaction's mortality and returns the first matching inclusion.
	///
	/// The search starts at `mortality.block_height` and proceeds one block at a time until the
	/// mortality period expires, optionally following the node's best chain when `use_best_block` is
	/// `true`.
	///
	/// # Returns
	/// - `Ok(Some(BlockInfo))` once an inclusion is confirmed or a higher nonce proves execution.
	/// - `Ok(None)` when the mortality period elapses without finding a match.
	/// - `Err(Error)` if block streaming or nonce queries fail.
	pub async fn find_correct_block_info(
		client: &Client,
		nonce: u32,
		tx_hash: H256,
		account_id: &AccountId,
		mortality: &RefinedMortality,
		use_best_block: bool,
	) -> Result<Option<BlockInfo>, Error> {
		let mortality_ends_height = mortality.block_height + mortality.period as u32;

		let mut sub = Sub::new(client.clone());
		sub.set_block_height(mortality.block_height);
		sub.use_best_block(use_best_block);

		let mut current_block_height = mortality.block_height;

		#[cfg(feature = "tracing")]
		{
			match use_best_block {
				true => {
					let info = client.best().block_info().await?;
					info!(target: "lib", "Nonce: {} Account address: {} Current Best Height: {} Mortality End Height: {}", nonce, account_id, info.height, mortality_ends_height);
				},
				false => {
					let info = client.finalized().block_info().await?;
					info!(target: "lib", "Nonce: {} Account address: {} Current Finalized Height: {} Mortality End Height: {}", nonce, account_id, info.height, mortality_ends_height);
				},
			};
		}

		while mortality_ends_height >= current_block_height {
			let info = sub.next().await?;
			current_block_height = info.height;

			let state_nonce = client.chain().block_nonce(account_id.clone(), info.hash).await?;
			if state_nonce > nonce {
				trace_new_block(nonce, state_nonce, account_id, info, true);
				return Ok(Some(info));
			}
			if state_nonce == 0 {
				let block = BlockApi::new(client.clone(), info.hash);
				let ext = block.raw_ext().get(tx_hash, EncodeSelector::None).await?;
				if ext.is_some() {
					trace_new_block(nonce, state_nonce, account_id, info, true);
					return Ok(Some(info));
				}
			}

			trace_new_block(nonce, state_nonce, account_id, info, false);
		}

		Ok(None)
	}
}

/// Emits optional tracing output detailing nonce progression while searching for a transaction.
///
/// When the `tracing` feature is disabled this function does nothing; otherwise it records each
/// inspected block along with whether the search completed.
fn trace_new_block(nonce: u32, state_nonce: u32, account_id: &AccountId, block_info: BlockInfo, search_done: bool) {
	#[cfg(feature = "tracing")]
	{
		if search_done {
			info!(target: "lib", "Account ({}, {}). At block ({}, {:?}) found nonce: {}. Search is done", nonce, account_id, block_info.height, block_info.hash, state_nonce);
		} else {
			info!(target: "lib", "Account ({}, {}). At block ({}, {:?}) found nonce: {}.", nonce, account_id, block_info.height, block_info.hash, state_nonce);
		}
	}

	#[cfg(not(feature = "tracing"))]
	{
		let _ = (nonce, state_nonce, account_id, block_info, search_done);
	}
}

/// Applies either the provided retry preference or the client's global retry configuration.
///
/// Returns `true` when retries should be attempted, `false` otherwise.
fn should_retry(client: &Client, value: Option<bool>) -> bool {
	value.unwrap_or(client.is_global_retries_enabled())
}
