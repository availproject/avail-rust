#[derive(Debug)]
#[repr(u8)]
pub enum ClientError {
	#[cfg(feature = "subxt")]
	Subxt(crate::subxt::Error) = 0,
	Core(core::Error) = 1,
	Custom(String) = 2,
}

impl From<core::Error> for ClientError {
	fn from(value: core::Error) -> Self {
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

#[cfg(feature = "subxt")]
impl From<crate::subxt::Error> for ClientError {
	fn from(value: crate::subxt::Error) -> Self {
		Self::Subxt(value)
	}
}
