#[derive(thiserror::Error, Debug)]
pub enum UserError {
	#[error("{0}")]
	Decoding(String),
	#[error("{0}")]
	ValidationFailed(String),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("{0}")]
	RpcError(avail_rust_core::rpc::Error),
	#[error("{0}")]
	User(UserError),
}

impl From<avail_rust_core::rpc::Error> for Error {
	fn from(value: avail_rust_core::rpc::Error) -> Self {
		Self::RpcError(value)
	}
}

impl From<UserError> for Error {
	fn from(value: UserError) -> Self {
		Self::User(value)
	}
}
