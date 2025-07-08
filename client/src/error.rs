use avail_rust_core::ext::codec;

#[derive(Debug)]
#[repr(u8)]
pub enum ClientError {
	#[cfg(feature = "subxt")]
	Subxt(crate::subxt::Error) = 0,
	Core(avail_rust_core::Error) = 1,
	Custom(String) = 2,
	Codec(codec::Error),
}

impl From<avail_rust_core::Error> for ClientError {
	fn from(value: avail_rust_core::Error) -> Self {
		Self::Core(value)
	}
}

impl From<String> for ClientError {
	fn from(value: String) -> Self {
		Self::Custom(value)
	}
}

impl From<&str> for ClientError {
	fn from(value: &str) -> Self {
		Self::Custom(String::from(value))
	}
}

impl From<codec::Error> for ClientError {
	fn from(value: codec::Error) -> Self {
		Self::Codec(value)
	}
}

#[cfg(feature = "subxt")]
impl From<crate::subxt::Error> for ClientError {
	fn from(value: crate::subxt::Error) -> Self {
		Self::Subxt(value)
	}
}
