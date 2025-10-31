use avail_rust_core::{AvailHeader, H256, HashNumber, rpc::ExtrinsicInfo, types::HashStringNumber};

use crate::{Client, Error, UserError, chain::Chain, conversions};

/// Fetches the block header for the provided identifier.
///
/// # Parameters
/// - `client`: RPC client used to perform the header query.
/// - `block_id`: Hash or number identifying the target block.
///
/// # Returns
/// - `Ok(AvailHeader)`: Header returned by the node.
/// - `Err(Error)`: The RPC call failed or the block could not be resolved.
///
/// # Side Effects
/// - Performs an RPC call through the client's chain interface.
#[derive(Clone)]
pub struct BlockContext {
	/// Client handle used for follow-up RPC calls.
	pub client: Client,
	/// Hash or number identifying the target block.
	pub block_id: HashStringNumber,
	retry_on_error: Option<bool>,
}

impl BlockContext {
	/// Creates a new block context for the provided identifier.
	///
	/// # Arguments
	/// * `client` - Client used to perform subsequent RPC calls.
	/// * `block_id` - Hash or number identifying the target block.
	pub fn new(client: Client, block_id: HashStringNumber) -> Self {
		Self { client, block_id, retry_on_error: None }
	}

	/// Overrides the retry policy for follow-up RPC calls.
	///
	/// # Arguments
	/// * `value` - `Some(true)` to force retries, `Some(false)` to disable retries, `None` to inherit defaults.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.retry_on_error = value;
	}

	/// Reports whether RPC calls should retry after errors.
	///
	/// # Returns
	/// Returns `true` when retries are enabled, otherwise `false`.
	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
	}

	/// Resolves the stored identifier into a [`HashNumber`].
	///
	/// # Returns
	/// Returns the resolved `HashNumber` or an error if conversion fails.
	pub fn hash_number(&self) -> Result<HashNumber, Error> {
		conversions::hash_string_number::to_hash_number(self.block_id.clone())
	}

	/// Returns a chain helper configured with the current retry policy.
	pub fn chain(&self) -> Chain {
		self.client.chain().retry_on(self.retry_on_error, None)
	}

	/// Fetches the block header associated with this context.
	///
	/// # Returns
	/// Returns the header or an error when the block cannot be resolved.
	pub async fn header(&self) -> Result<AvailHeader, Error> {
		let header = self.chain().block_header(Some(self.block_id.clone())).await?;
		let Some(header) = header else {
			return Err(Error::User(UserError::Other(std::format!(
				"No block header found for block id: {}",
				self.block_id
			))));
		};

		Ok(header)
	}

	/// Counts the events emitted by the block referenced by this context.
	///
	/// # Returns
	/// Returns the number of events or an error when the RPC call fails.
	pub async fn event_count(&self) -> Result<usize, Error> {
		self.chain().block_event_count(self.block_id.clone()).await
	}
}

/// Metadata describing where an extrinsic resides within a block.
#[derive(Debug, Clone)]
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
	pub block_id: HashNumber,
}

impl BlockExtrinsicMetadata {
	/// Wraps metadata about an extrinsic inside a block.
	///
	/// # Parameters
	/// - `ext_hash`: Hash of the extrinsic.
	/// - `ext_index`: Index of the extrinsic within the block.
	/// - `pallet_id`: Pallet identifier associated with the call.
	/// - `variant_id`: Variant identifier within the pallet.
	/// - `block_id`: Hash or number of the block containing the extrinsic.
	///
	/// # Returns
	/// - `Self`: Metadata wrapper encapsulating the supplied values.
	pub fn new(ext_hash: H256, ext_index: u32, pallet_id: u8, variant_id: u8, block_id: HashNumber) -> Self {
		Self { ext_hash, ext_index, pallet_id, variant_id, block_id }
	}

	/// Builds metadata from RPC extrinsic information.
	///
	/// # Arguments
	/// * `info` - RPC result describing an extrinsic.
	/// * `block_id` - Hash or number identifying the block containing the extrinsic.
	///
	/// # Returns
	/// Returns a metadata wrapper encapsulating the provided information.
	pub fn from_extrinsic_info(info: &ExtrinsicInfo, block_id: HashNumber) -> Self {
		Self::new(info.ext_hash, info.ext_index, info.pallet_id, info.variant_id, block_id)
	}
}
