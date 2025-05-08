use thiserror::Error;

#[derive(Error, Debug)]
#[repr(u8)]
pub enum RpcError {
	#[cfg(feature = "subxt")]
	#[error("{0}")]
	Subxt(subxt::Error) = 0,
	#[error("{0}")]
	SubxtCore(subxt_core::Error) = 1,
	#[error("{0}")]
	SubxtRpcs(subxt_rpcs::Error) = 2,
	#[error("{0}")]
	Custom(String) = 3,
	#[error("Transaction is not allowed. {0}")]
	TransactionNotAllowed(String) = 4,
}

impl From<subxt_core::Error> for RpcError {
	fn from(value: subxt_core::Error) -> Self {
		Self::SubxtCore(value)
	}
}

#[cfg(feature = "subxt")]
impl From<subxt::Error> for RpcError {
	fn from(value: subxt::Error) -> Self {
		Self::Subxt(value)
	}
}

impl From<subxt_rpcs::Error> for RpcError {
	fn from(value: subxt_rpcs::Error) -> Self {
		Self::SubxtRpcs(value)
	}
}

impl From<String> for RpcError {
	fn from(value: String) -> Self {
		Self::Custom(value)
	}
}

impl From<&str> for RpcError {
	fn from(value: &str) -> Self {
		Self::Custom(String::from(value))
	}
}

#[derive(Error, Debug)]
#[repr(u8)]
pub enum ClientError {
	#[cfg(feature = "subxt")]
	#[error("Subxt error. {0}")]
	Subxt(subxt::Error) = 0,
	#[error("Rpc error. {0}")]
	RpcError(RpcError) = 1,
}

impl From<RpcError> for ClientError {
	fn from(value: RpcError) -> Self {
		Self::RpcError(value)
	}
}

#[cfg(feature = "subxt")]
impl From<subxt::Error> for ClientError {
	fn from(value: subxt::Error) -> Self {
		Self::Subxt(value)
	}
}
