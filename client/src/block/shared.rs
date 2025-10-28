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
pub struct BlockContext {
	pub client: Client,
	pub block_id: HashStringNumber,
	retry_on_error: Option<bool>,
}

impl BlockContext {
	pub fn new(client: Client, block_id: HashStringNumber) -> Self {
		Self { client, block_id, retry_on_error: None }
	}

	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.retry_on_error = value;
	}

	pub fn should_retry_on_error(&self) -> bool {
		self.retry_on_error
			.unwrap_or_else(|| self.client.is_global_retries_enabled())
	}

	pub fn hash_number(&self) -> Result<HashNumber, Error> {
		conversions::hash_string_number::to_hash_number(self.block_id.clone())
	}

	pub fn chain(&self) -> Chain {
		self.client.chain().retry_on(self.retry_on_error, None)
	}

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

	pub async fn event_count(&self) -> Result<usize, Error> {
		self.chain().block_event_count(self.block_id.clone()).await
	}
}

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

	pub fn from_extrinsic_info(info: &ExtrinsicInfo, block_id: HashNumber) -> Self {
		Self::new(info.ext_hash, info.ext_index, info.pallet_id, info.variant_id, block_id)
	}
}
