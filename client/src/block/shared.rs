use avail_rust_core::{
	AvailHeader, H256, HashNumber,
	rpc::{self},
	types::HashStringNumber,
};

use crate::{Client, Error, RetryPolicy, chain::Chain, error_ops};

/// Shared block query context used by block helpers.
#[derive(Clone)]
pub struct BlockContext {
	/// Client handle used for follow-up RPC calls.
	pub client: Client,
	/// Hash or number identifying the target block.
	pub at: HashStringNumber,
	retry_on_error: RetryPolicy,
}

impl BlockContext {
	/// Creates a new block context for a hash/height identifier.
	pub fn new(client: Client, at: HashStringNumber) -> Self {
		Self { client, at, retry_on_error: RetryPolicy::Inherit }
	}

	/// Sets retry behavior for follow-up RPC calls.
	pub fn set_retry_policy(&mut self, value: RetryPolicy) {
		self.retry_on_error = value;
	}

	/// Returns whether RPC calls retry after errors.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.resolve(self.client.retry_policy() != RetryPolicy::Disabled)
	}

	/// Returns effective retry policy (`Enabled` or `Disabled`).
	pub fn retry_policy(&self) -> RetryPolicy {
		if self.should_retry_on_error() {
			RetryPolicy::Enabled
		} else {
			RetryPolicy::Disabled
		}
	}

	/// Resolves the stored identifier into a [`HashNumber`].
	pub fn hash_number(&self) -> Result<HashNumber, Error> {
		HashNumber::from_impl(self.at.clone())
			.map_err(|e| Error::validation_with_op(error_ops::ErrorOperation::BlockSharedHashNumber, e))
	}

	/// Returns a chain helper configured with the current retry policy.
	pub fn chain(&self) -> Chain {
		self.client
			.chain()
			.retry_policy(self.retry_on_error, RetryPolicy::Inherit)
	}

	/// Fetches the block header associated with this context.
	pub async fn header(&self) -> Result<AvailHeader, Error> {
		let header = self.chain().block_header(Some(self.at.clone())).await?;
		let Some(header) = header else {
			return Err(Error::not_found_with_op(
				error_ops::ErrorOperation::BlockSharedHeader,
				std::format!("No block header found for block id: {}", self.at),
			));
		};

		Ok(header)
	}

	/// Counts the events emitted by the block referenced by this context.
	///
	/// Returns the number of events or an error when the RPC call fails.
	pub async fn event_count(&self) -> Result<usize, Error> {
		self.chain().block_event_count(self.at.clone()).await
	}
}

/// Metadata describing where an extrinsic resides within a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockExtrinsicMetadata {
	/// Hash of the extrinsic.
	pub ext_hash: H256,
	/// Index of the extrinsic within the block.
	pub ext_index: u32,
	/// Pallet identifier associated with the call.
	pub pallet_id: u8,
	/// Variant within the pallet identifying the call.
	pub variant_id: u8,
	/// Block identifier (hash or number) where the extrinsic resides.
	pub at: HashNumber,
}

impl BlockExtrinsicMetadata {
	/// Wraps metadata about an extrinsic inside a block.
	///
	pub fn new(ext_hash: H256, ext_index: u32, pallet_id: u8, variant_id: u8, at: HashNumber) -> Self {
		Self { ext_hash, ext_index, pallet_id, variant_id, at }
	}

	/// Builds metadata from RPC extrinsic information.
	///
	/// Returns a metadata wrapper encapsulating the provided information.
	pub fn from_rpc_extrinsic(ext: &rpc::Extrinsic, at: HashNumber) -> Self {
		Self::new(ext.ext_hash, ext.ext_index, ext.pallet_id, ext.variant_id, at)
	}
}
