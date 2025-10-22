use avail_rust_core::{AvailHeader, HashNumber, types::HashStringNumber};

use crate::{Client, Error, UserError, chain::Chain};

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
pub async fn header(client: &Client, block_id: HashStringNumber) -> Result<AvailHeader, Error> {
	let header = client.chain().block_header(Some(block_id.clone())).await?;
	let Some(header) = header else {
		return Err(Error::User(UserError::Other(std::format!("No block header found for block id: {}", block_id))));
	};

	Ok(header)
}

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
		Ok(self.block_id.clone().try_into().map_err(UserError::Decoding)?)
	}

	pub fn chain(&self) -> Chain {
		self.client.chain().retry_on(self.retry_on_error, None)
	}
}
