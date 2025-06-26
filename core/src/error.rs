#[derive(thiserror::Error, Debug)]
#[repr(u8)]
pub enum Error {
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

impl From<subxt_core::Error> for Error {
	fn from(value: subxt_core::Error) -> Self {
		Self::SubxtCore(value)
	}
}

#[cfg(feature = "subxt")]
impl From<subxt::Error> for Error {
	fn from(value: subxt::Error) -> Self {
		Self::Subxt(value)
	}
}

impl From<subxt_rpcs::Error> for Error {
	fn from(value: subxt_rpcs::Error) -> Self {
		Self::SubxtRpcs(value)
	}
}

impl From<String> for Error {
	fn from(value: String) -> Self {
		Self::Custom(value)
	}
}

impl From<&str> for Error {
	fn from(value: &str) -> Self {
		Self::Custom(String::from(value))
	}
}
