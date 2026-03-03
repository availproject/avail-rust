use super::submitted::SubmissionOutcome;
use crate::{
	Client, Error, RetryPolicy, chain::Chain, submission::submitted::WaitOption, subxt_signer::sr25519::Keypair,
	transaction_options::Options,
};
use avail_rust_core::{
	ExtrinsicBorrowed, H256, HasHeader, RpcError,
	ext::codec::Encode,
	substrate::extrinsic::ExtrinsicCall,
	types::substrate::{FeeDetails, RuntimeDispatchInfo},
};

/// Builder that keeps an encoded call together with the client connection and exposes helpers for
/// signing, submitting, and querying execution costs.
#[derive(Clone)]
pub struct SubmittableTransaction {
	client: Client,
	pub call: ExtrinsicCall,
	retry_on_error: RetryPolicy,
}

impl SubmittableTransaction {
	/// Creates a transaction builder from an encoded call.
	///
	/// The builder is inert until one of the async helpers is invoked. By default it inherits the
	/// client's retry policy, but this can be customised via [`set_retry_policy`](Self::set_retry_policy).
	pub fn new(client: Client, call: ExtrinsicCall) -> Self {
		Self { client, call, retry_on_error: RetryPolicy::Inherit }
	}

	/// Signs the call with the provided keypair and submits it to the chain in a single RPC round-trip.
	///
	///   metadata inferred from `options`.
	///   (potentially retried according to the configured policy) returns an error.
	///
	/// The submission uses `options` (nonce, tip, mortality) exactly as provided; no additional mutation
	/// happens inside this helper.
	pub async fn submit(&self, signer: &Keypair, options: Options) -> Result<super::SubmittedTransaction, Error> {
		self.chain().sign_and_submit_call(signer, &self.call.0, options).await
	}

	pub async fn submit_and_wait_for_receipt(
		&self,
		signer: &Keypair,
		options: Options,
		wait_opts: impl Into<WaitOption>,
	) -> Result<super::TransactionReceipt, Error> {
		let submitted = self.submit(signer, options).await?;
		submitted.receipt(wait_opts).await
	}

	pub async fn submit_and_wait_for_outcome(
		&self,
		signer: &Keypair,
		options: Options,
		wait_opts: impl Into<WaitOption>,
	) -> Result<SubmissionOutcome, Error> {
		let submitted = self.submit(signer, options).await?;
		submitted.outcome(wait_opts).await
	}

	pub async fn sign<'a>(&'a self, signer: &Keypair, options: Options) -> Result<ExtrinsicBorrowed<'a>, Error> {
		self.chain()
			.build_extrinsic_from_call(signer, &self.call.0, options)
			.await
	}

	/// Estimates call fees without signing or submitting.
	/// Returns an RPC error when fee simulation fails.
	pub async fn estimate_call_fees(&self, at: Option<H256>) -> Result<FeeDetails, RpcError> {
		let call = self.call.encode();
		self.chain().transaction_payment_query_call_fee_details(call, at).await
	}

	/// Signs the call and estimates fees for the exact extrinsic payload.
	pub async fn estimate_extrinsic_fees(
		&self,
		signer: &Keypair,
		options: Options,
		at: Option<H256>,
	) -> Result<FeeDetails, Error> {
		let transaction = self.sign(signer, options).await?;
		let transaction = transaction.encode();
		Ok(self
			.chain()
			.transaction_payment_query_fee_details(transaction, at)
			.await?)
	}

	/// Returns runtime dispatch information for the call, including weight, class, and partial fee
	/// estimation based on the provided block context.
	///
	///   transport failure).
	pub async fn call_info(&self, at: Option<H256>) -> Result<RuntimeDispatchInfo, RpcError> {
		let call = self.call.encode();
		self.chain().transaction_payment_query_call_info(call, at).await
	}

	/// Resolves whether RPC calls performed through this builder should be retried on transient
	/// failures.
	///
	/// The method returns the explicit override set by [`set_retry_policy`](Self::set_retry_policy),
	/// falling back to the client's global retry configuration when no override is present.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.resolve(self.client.retry_policy() != RetryPolicy::Disabled)
	}

	/// Controls retry behaviour for RPC calls sent via this builder.
	///
	/// - [`RetryPolicy::Enabled`]: force retries regardless of the client's global setting.
	/// - [`RetryPolicy::Disabled`]: disable retries for requests issued through this builder.
	/// - [`RetryPolicy::Inherit`]: fall back to the client's global retry configuration.
	pub fn set_retry_policy(&mut self, value: RetryPolicy) {
		self.retry_on_error = value;
	}

	/// Converts any encodable call into a `SubmittableTransaction` based on its pallet and call indices.
	/// The provided value is SCALE-encoded immediately; failures propagate as panics originating from
	/// the underlying encoding implementation.
	pub fn from_encodable<T: HasHeader + Encode>(client: Client, value: T) -> SubmittableTransaction {
		let call = ExtrinsicCall::from_parts(T::HEADER_INDEX.0, T::HEADER_INDEX.1, value.encode());
		SubmittableTransaction::new(client, call)
	}

	/// Hashes the call payload as it would appear in an extrinsic, returning the blake2 hash used by
	/// the runtime for call identification.
	pub fn call_hash(&self) -> H256 {
		H256::from(self.call.hash())
	}

	fn chain(&self) -> Chain {
		self.client
			.chain()
			.retry_policy(self.retry_on_error, RetryPolicy::Inherit)
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
