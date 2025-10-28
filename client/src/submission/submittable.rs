use crate::{Client, Error, subxt_signer::sr25519::Keypair, transaction_options::Options};
use avail_rust_core::{
	H256, HasHeader, RpcError,
	ext::codec::Encode,
	substrate::extrinsic::{ExtrinsicCall, GenericExtrinsic},
	types::substrate::{FeeDetails, RuntimeDispatchInfo},
};

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
	pub async fn sign_and_submit(
		&self,
		signer: &Keypair,
		options: Options,
	) -> Result<super::SubmittedTransaction, Error> {
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
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
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
	pub fn call_hash(&self) -> H256 {
		H256::from(self.call.hash())
	}
}

impl From<SubmittableTransaction> for ExtrinsicCall {
	fn from(value: SubmittableTransaction) -> Self {
		value.call
	}
}

impl From<&SubmittableTransaction> for ExtrinsicCall {
	fn from(value: &SubmittableTransaction) -> Self {
		value.call.clone()
	}
}
