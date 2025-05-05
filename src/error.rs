#[derive(Debug)]
pub enum RpcError {
	Subxt(subxt::Error),
	SubxtCore(subxt_core::Error),
	SubxtRpcs(subxt_rpcs::Error),
	Custom(String),
	TransactionNotAllowed(String),
}

impl RpcError {
	pub fn to_string(&self) -> String {
		match self {
			Self::Subxt(e) => e.to_string(),
			Self::SubxtCore(e) => e.to_string(),
			Self::SubxtRpcs(e) => e.to_string(),
			Self::Custom(e) => e.to_string(),
			Self::TransactionNotAllowed(e) => e.to_string(),
		}
	}
}

impl From<subxt_core::Error> for RpcError {
	fn from(value: subxt_core::Error) -> Self {
		Self::SubxtCore(value)
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
	Subxt(subxt::Error),
	RpcError(RpcError),
}

impl ClientError {
	pub fn to_string(&self) -> String {
		match self {
			Self::Custom(e) => e.clone(),
			Self::Subxt(e) => e.to_string(),
			Self::RpcError(e) => e.to_string(),
		}
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
