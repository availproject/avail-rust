use avail_rust_core::{
	EncodeSelector,
	rpc::{self, ExtrinsicFilter},
};

#[derive(Debug, Default, Clone)]
pub struct Options {
	pub filter: Option<ExtrinsicFilter>,
	pub ss58_address: Option<String>,
	pub app_id: Option<u32>,
	pub nonce: Option<u32>,
}

impl Options {
	/// Creates a builder with all filters unset.
	///
	/// # Returns
	/// - `Self`: Options builder with default values.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the expected nonce filter.
	///
	/// # Parameters
	/// - `value`: Nonce that matching extrinsics must carry.
	///
	/// # Returns
	/// - `Self`: Builder with the nonce filter applied.
	pub fn nonce(mut self, value: u32) -> Self {
		self.nonce = Some(value);
		self
	}

	/// Sets the application identifier filter.
	///
	/// # Parameters
	/// - `value`: Application identifier required for matching extrinsics.
	///
	/// # Returns
	/// - `Self`: Builder with the application filter applied.
	pub fn app_id(mut self, value: u32) -> Self {
		self.app_id = Some(value);
		self
	}

	/// Sets the signer address filter.
	///
	/// # Parameters
	/// - `value`: Address (SS58 format) required for matching extrinsics.
	///
	/// # Returns
	/// - `Self`: Builder with the address filter applied.
	pub fn ss58_address(mut self, value: impl Into<String>) -> Self {
		self.ss58_address = Some(value.into());
		self
	}

	/// Sets the primary transaction filter.
	///
	/// # Parameters
	/// - `value`: Filter describing the target extrinsics (hash, index, or number).
	///
	/// # Returns
	/// - `Self`: Builder with the transaction filter applied.
	pub fn filter(mut self, value: impl Into<ExtrinsicFilter>) -> Self {
		self.filter = Some(value.into());
		self
	}

	/// Converts the builder into RPC options with the requested encoding.
	///
	/// # Parameters
	/// - `encode_as`: Encoding preference for the RPC response.
	///
	/// # Returns
	/// - `rpc::ExtrinsicOpts`: Ready-to-send RPC configuration.
	pub fn to_rpc_opts(self, encode_as: EncodeSelector) -> rpc::ExtrinsicOpts {
		rpc::ExtrinsicOpts {
			transaction_filter: self.filter.unwrap_or_default(),
			ss58_address: self.ss58_address,
			app_id: self.app_id,
			nonce: self.nonce,
			encode_as,
		}
	}
}
