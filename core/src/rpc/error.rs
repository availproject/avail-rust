#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("{0}")]
	Rpc(subxt_rpcs::Error),
	#[error("RPC error: cannot decode some part of the response. Response might be malformed: {0}")]
	MalformedResponse(String),
	#[error("RPC error: cannot decode some part of the response: {0}")]
	DecodingFailed(String),
	#[error("RPC error: expected to receive data but not data was received: {0}")]
	ExpectedData(String),
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
