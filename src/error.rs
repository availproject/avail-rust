use subxt::error::DispatchError;
use subxt_signer::{sr25519, SecretUriError};

#[derive(Debug)]
pub enum RpcError {
	Subxt(subxt::Error),
	SubxtRpcs(subxt_rpcs::Error),
	Custom(String),
}

impl RpcError {
	pub fn to_string(&self) -> String {
		match self {
			Self::Subxt(e) => e.to_string(),
			Self::SubxtRpcs(e) => e.to_string(),
			Self::Custom(e) => e.to_string(),
		}
	}
}

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

#[derive(Debug)]
pub enum ClientError {
	Custom(String),
	SerdeJson(serde_json::Error),
	Subxt(subxt::Error),
	SubxtCore(subxt_core::Error),
	SubxtSigner(SecretUriError),
	Sr25519(sr25519::Error),
	RpcError(RpcError),
}

impl ClientError {
	pub fn to_string(&self) -> String {
		match self {
			Self::Custom(e) => e.clone(),
			Self::SerdeJson(e) => e.to_string(),
			Self::Subxt(e) => e.to_string(),
			Self::SubxtCore(e) => e.to_string(),
			Self::SubxtSigner(e) => e.to_string(),
			Self::Sr25519(e) => e.to_string(),
			Self::RpcError(e) => e.to_string(),
		}
	}
}

impl From<subxt_rpcs::Error> for ClientError {
	fn from(value: subxt_rpcs::Error) -> Self {
		Self::RpcError(RpcError::SubxtRpcs(value))
	}
}

impl From<RpcError> for ClientError {
	fn from(value: RpcError) -> Self {
		Self::RpcError(value)
	}
}

impl From<&str> for ClientError {
	fn from(value: &str) -> Self {
		Self::Custom(value.to_string())
	}
}

impl From<String> for ClientError {
	fn from(value: String) -> Self {
		Self::Custom(value.to_string())
	}
}

impl From<subxt::Error> for ClientError {
	fn from(value: subxt::Error) -> Self {
		Self::Subxt(value)
	}
}

impl From<subxt_core::Error> for ClientError {
	fn from(value: subxt_core::Error) -> Self {
		Self::SubxtCore(value)
	}
}

impl From<DispatchError> for ClientError {
	fn from(value: DispatchError) -> Self {
		Self::Subxt(value.into())
	}
}

impl From<SecretUriError> for ClientError {
	fn from(value: SecretUriError) -> Self {
		Self::SubxtSigner(value)
	}
}

impl From<sr25519::Error> for ClientError {
	fn from(value: sr25519::Error) -> Self {
		Self::Sr25519(value)
	}
}

impl From<serde_json::Error> for ClientError {
	fn from(value: serde_json::Error) -> Self {
		Self::SerdeJson(value)
	}
}
