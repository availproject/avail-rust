/// Errors that originate from user input or validation problems.
#[derive(thiserror::Error, Debug)]
pub enum UserError {
	/// Indicates SCALE or JSON decoding failed.
	#[error("{0}")]
	Decoding(String),
	/// Indicates validation rules were violated.
	#[error("{0}")]
	ValidationFailed(String),
	/// Catch-all for other user-facing errors.
	#[error("{0}")]
	Other(String),
}

/// Errors raised by the Avail client.
#[derive(thiserror::Error, Debug)]
pub enum Error {
	/// Wraps lower-level RPC errors propagated from the transport layer.
	#[error("{0}")]
	RpcError(avail_rust_core::rpc::Error),
	/// Wraps `UserError` variants.
	#[error("{0}")]
	User(UserError),
	/// Catch-all for other error conditions.
	#[error("{0}")]
	Other(String),
}

impl From<avail_rust_core::rpc::Error> for Error {
	/// Converts a core RPC error into the client error type.
	fn from(value: avail_rust_core::rpc::Error) -> Self {
		Self::RpcError(value)
	}
}

impl From<UserError> for Error {
	/// Wraps a `UserError` into the unified error type.
	fn from(value: UserError) -> Self {
		Self::User(value)
	}
}

impl From<&str> for Error {
	/// Converts a string slice into a generic error variant.
	fn from(value: &str) -> Self {
		Self::Other(value.to_owned())
	}
}

impl From<codec::Error> for Error {
	/// Converts SCALE codec errors into the client error type.
	fn from(value: codec::Error) -> Self {
		Self::Other(value.to_string())
	}
}
