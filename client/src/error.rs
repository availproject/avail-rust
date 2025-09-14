#[derive(Debug)]
pub enum UserError {
	Decoding(String),
	ValidationFailed(String),
}

#[derive(Debug)]
pub enum Error {
	RpcError(avail_rust_core::rpc::Error),
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
