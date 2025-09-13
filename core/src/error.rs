use crate::rpc;

#[derive(thiserror::Error, Debug)]
#[repr(u8)]
pub enum Error {
	#[error("{0}")]
	Rpc(rpc::Error),
	#[cfg(feature = "subxt")]
	#[error("{0}")]
	Subxt(subxt::Error),
	#[error("Transaction is not allowed. {0}")]
	TransactionNotAllowed(String),
}

impl From<rpc::Error> for Error {
	fn from(value: rpc::Error) -> Self {
		Self::Rpc(value)
	}
}

/* impl From<subxt_core::Error> for Error {
	fn from(value: subxt_core::Error) -> Self {
		Self::SubxtCore(value)
	}
}
 */
#[cfg(feature = "subxt")]
impl From<subxt::Error> for Error {
	fn from(value: subxt::Error) -> Self {
		Self::Subxt(value)
	}
}

/* impl From<subxt_rpcs::Error> for Error {
	fn from(value: subxt_rpcs::Error) -> Self {
		Self::Rpc(value)
	}
} */

/* impl From<String> for Error {
	fn from(value: String) -> Self {
		Self::Custom(value)
	}
}

impl From<&str> for Error {
	fn from(value: &str) -> Self {
		Self::Custom(String::from(value))
	}
}

impl From<codec::Error> for Error {
	fn from(value: codec::Error) -> Self {
		Self::Codec(value)
	}
}

impl From<const_hex::FromHexError> for Error {
	fn from(value: const_hex::FromHexError) -> Self {
		Self::FromHex(value)
	}
}
 */
