use crate::error_ops::ErrorOperation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
	Validation,
	Transport,
	Rpc,
	Timeout,
	NotFound,
	Decode,
	Other,
}

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
	#[error("{0}")]
	Validation(String),
	#[error("{0}")]
	Transport(String),
	#[error("{0}")]
	Timeout(String),
	#[error("{0}")]
	NotFound(String),
	#[error("{0}")]
	Decode(String),
	#[error("{0}")]
	Rpc(String),
	/// Wraps `UserError` variants.
	#[error("{0}")]
	User(UserError),
	/// Catch-all for other error conditions.
	#[error("{0}")]
	Other(String),
}

impl Error {
	fn op_message(operation: ErrorOperation, message: impl Into<String>) -> String {
		std::format!("[op:{}] {}", operation.as_str(), message.into())
	}

	pub fn validation_with_op(operation: ErrorOperation, message: impl Into<String>) -> Self {
		Self::Validation(Self::op_message(operation, message))
	}

	pub fn not_found_with_op(operation: ErrorOperation, message: impl Into<String>) -> Self {
		Self::NotFound(Self::op_message(operation, message))
	}

	pub fn decode_with_op(operation: ErrorOperation, message: impl Into<String>) -> Self {
		Self::Decode(Self::op_message(operation, message))
	}

	pub fn transport_with_op(operation: ErrorOperation, message: impl Into<String>) -> Self {
		Self::Transport(Self::op_message(operation, message))
	}

	pub fn rpc_with_op(operation: ErrorOperation, message: impl Into<String>) -> Self {
		Self::Rpc(Self::op_message(operation, message))
	}

	pub fn operation_code(&self) -> Option<ErrorOperation> {
		fn parse(input: &str) -> Option<ErrorOperation> {
			if !input.starts_with("[op:") {
				return None;
			}
			let end = input.find(']')?;
			ErrorOperation::parse(&input[4..end])
		}

		match self {
			Error::Validation(msg)
			| Error::Transport(msg)
			| Error::Timeout(msg)
			| Error::NotFound(msg)
			| Error::Decode(msg)
			| Error::Rpc(msg)
			| Error::Other(msg) => parse(msg),
			Error::User(UserError::Decoding(msg))
			| Error::User(UserError::ValidationFailed(msg))
			| Error::User(UserError::Other(msg)) => parse(msg),
		}
	}

	pub fn code(&self) -> ErrorCode {
		match self {
			Error::Validation(_) => ErrorCode::Validation,
			Error::Transport(_) => ErrorCode::Transport,
			Error::Timeout(_) => ErrorCode::Timeout,
			Error::NotFound(_) => ErrorCode::NotFound,
			Error::Decode(_) => ErrorCode::Decode,
			Error::Rpc(_) => ErrorCode::Rpc,
			Error::User(inner) => match inner {
				UserError::Decoding(_) => ErrorCode::Decode,
				UserError::ValidationFailed(_) => ErrorCode::Validation,
				UserError::Other(_) => ErrorCode::Other,
			},
			Error::Other(_) => ErrorCode::Other,
		}
	}
}

impl From<avail_rust_core::rpc::Error> for Error {
	/// Converts a core RPC error into the client error type.
	fn from(value: avail_rust_core::rpc::Error) -> Self {
		match value {
			avail_rust_core::rpc::Error::DecodingFailed(msg) => Self::Decode(msg),
			avail_rust_core::rpc::Error::MalformedResponse(msg) => Self::Decode(msg),
			avail_rust_core::rpc::Error::ExpectedData(msg) => Self::NotFound(msg),
			avail_rust_core::rpc::Error::UnexpectedInput(msg) => Self::Validation(msg),
			avail_rust_core::rpc::Error::Rpc(inner) => Self::Rpc(inner.to_string()),
		}
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
		Self::Decode(value.to_string())
	}
}
