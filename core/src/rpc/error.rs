#[derive(thiserror::Error, Debug)]
#[repr(u8)]
pub enum Error {
	#[error("{0}")]
	Rpc(subxt_rpcs::Error),
	#[error("RPC error: cannot decode some part of the response: {0}")]
	MalformedResponse(String),
	#[error("RPC error: cannot decode some part of the response: {0}")]
	DecodingFailed(String),
}

impl Error {
	pub fn malformed_response(value: impl Into<String>) -> Self {
		Self::MalformedResponse(value.into())
	}

	pub fn decoding_failed(value: impl Into<String>) -> Self {
		Self::DecodingFailed(value.into())
	}
}

impl From<subxt_rpcs::Error> for Error {
	fn from(value: subxt_rpcs::Error) -> Self {
		Self::Rpc(value)
	}
}

impl From<const_hex::FromHexError> for Error {
	fn from(value: const_hex::FromHexError) -> Self {
		Self::MalformedResponse(value.to_string())
	}
}
