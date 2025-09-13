use codec::{Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use subxt_core::utils::AccountId32;

pub type AccountId = AccountId32;
pub type AccountIndex = u32;
pub type BlockHeight = u32;
pub type BlockHash = H256;
pub type Signature = MultiSignature;
pub type BlakeTwo256 = subxt_core::config::substrate::BlakeTwo256;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, scale_info::TypeInfo)]
#[repr(u8)]
pub enum MultiSignature {
	/// An Ed25519 signature.
	Ed25519([u8; 64]) = 0,
	/// An Sr25519 signature.
	Sr25519([u8; 64]) = 1,
	/// An ECDSA/SECP256k1 signature (a 512-bit value, plus 8 bits for recovery ID).
	Ecdsa([u8; 65]) = 2,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, scale_info::TypeInfo)]
#[repr(u8)]
pub enum MultiAddress {
	/// It's an account ID (pubkey).
	Id(AccountId) = 0,
	/// It's an account index.
	Index(#[codec(compact)] u32) = 1,
	/// It's some arbitrary raw bytes.
	Raw(Vec<u8>) = 2,
	/// It's a 32 byte representation.
	Address32([u8; 32]) = 3,
	/// Its a 20 byte representation.
	Address20([u8; 20]) = 4,
}

impl From<AccountId> for MultiAddress {
	fn from(a: AccountId) -> Self {
		Self::Id(a)
	}
}

#[derive(Debug, Clone, Copy, Default, Encode, Decode, Eq, PartialEq)]
pub struct AppId(#[codec(compact)] pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct BlockRef {
	pub hash: H256,
	pub height: u32,
}

impl From<(H256, u32)> for BlockRef {
	fn from(value: (H256, u32)) -> Self {
		Self { hash: value.0, height: value.1 }
	}
}

impl From<BlockRef> for H256 {
	fn from(value: BlockRef) -> Self {
		value.hash
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct TxRef {
	pub hash: H256,
	pub index: u32,
}

impl From<(H256, u32)> for TxRef {
	fn from(value: (H256, u32)) -> Self {
		Self { hash: value.0, index: value.1 }
	}
}

impl From<TxRef> for H256 {
	fn from(value: TxRef) -> Self {
		value.hash
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HashNumber {
	Hash(H256),
	Number(u32),
}

impl From<BlockRef> for HashNumber {
	fn from(value: BlockRef) -> Self {
		Self::Hash(value.hash)
	}
}

impl From<TxRef> for HashNumber {
	fn from(value: TxRef) -> Self {
		Self::Number(value.index)
	}
}

impl From<H256> for HashNumber {
	fn from(value: H256) -> Self {
		Self::Hash(value)
	}
}

impl From<u32> for HashNumber {
	fn from(value: u32) -> Self {
		Self::Number(value)
	}
}

impl TryFrom<&str> for HashNumber {
	type Error = crate::Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Ok(Self::Hash(H256::from_str(value).map_err(|e| e.to_string())?))
	}
}

impl TryFrom<String> for HashNumber {
	type Error = crate::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		HashNumber::try_from(value.as_str())
	}
}

impl TryFrom<HashStringNumber> for HashNumber {
	type Error = crate::Error;

	fn try_from(value: HashStringNumber) -> Result<Self, Self::Error> {
		let v = match value {
			HashStringNumber::Hash(x) => Self::Hash(x),
			HashStringNumber::String(x) => {
				let hash = H256::from_str(&x).map_err(|x| x.to_string())?;
				Self::Hash(hash)
			},
			HashStringNumber::Number(x) => Self::Number(x),
		};
		Ok(v)
	}
}

impl TryFrom<HashString> for HashNumber {
	type Error = crate::Error;

	fn try_from(value: HashString) -> Result<Self, Self::Error> {
		Self::try_from(HashStringNumber::from(value))
	}
}

#[derive(Debug, Clone)]
pub enum HashString {
	Hash(H256),
	String(String),
}

impl TryInto<H256> for HashString {
	type Error = crate::Error;

	fn try_into(self) -> Result<H256, Self::Error> {
		match self {
			HashString::Hash(x) => Ok(x),
			HashString::String(x) => Ok(H256::from_str(&x).map_err(|x| x.to_string())?),
		}
	}
}

impl From<H256> for HashString {
	fn from(value: H256) -> Self {
		Self::Hash(value)
	}
}

impl From<&str> for HashString {
	fn from(value: &str) -> Self {
		Self::String(value.to_owned())
	}
}

impl From<&String> for HashString {
	fn from(value: &String) -> Self {
		Self::String(value.clone())
	}
}

impl From<String> for HashString {
	fn from(value: String) -> Self {
		Self::String(value)
	}
}

#[derive(Debug, Clone)]
pub enum HashStringNumber {
	Hash(H256),
	String(String),
	Number(u32),
}

impl From<BlockRef> for HashStringNumber {
	fn from(value: BlockRef) -> Self {
		Self::Hash(value.hash)
	}
}

impl From<HashString> for HashStringNumber {
	fn from(value: HashString) -> Self {
		match value {
			HashString::Hash(x) => Self::Hash(x),
			HashString::String(x) => Self::String(x),
		}
	}
}

impl From<HashNumber> for HashStringNumber {
	fn from(value: HashNumber) -> Self {
		match value {
			HashNumber::Hash(x) => Self::Hash(x),
			HashNumber::Number(x) => Self::Number(x),
		}
	}
}

impl From<TxRef> for HashStringNumber {
	fn from(value: TxRef) -> Self {
		Self::Number(value.index)
	}
}

impl From<H256> for HashStringNumber {
	fn from(value: H256) -> Self {
		Self::Hash(value)
	}
}

impl From<u32> for HashStringNumber {
	fn from(value: u32) -> Self {
		Self::Number(value)
	}
}

impl From<&str> for HashStringNumber {
	fn from(value: &str) -> Self {
		Self::String(value.to_owned())
	}
}

impl From<&String> for HashStringNumber {
	fn from(value: &String) -> Self {
		Self::String(value.clone())
	}
}

impl From<String> for HashStringNumber {
	fn from(value: String) -> Self {
		Self::String(value)
	}
}

#[derive(Debug, Clone)]
pub enum StringOrBytes<'a> {
	String(&'a str),
	Bytes(&'a [u8]),
}

impl<'a> From<&'a String> for StringOrBytes<'a> {
	fn from(value: &'a String) -> Self {
		Self::String(value.as_str())
	}
}

impl<'a> From<&'a str> for StringOrBytes<'a> {
	fn from(value: &'a str) -> Self {
		Self::String(value)
	}
}

impl<'a> From<&'a Vec<u8>> for StringOrBytes<'a> {
	fn from(value: &'a Vec<u8>) -> Self {
		Self::Bytes(value.as_slice())
	}
}

impl<'a> From<&'a [u8]> for StringOrBytes<'a> {
	fn from(value: &'a [u8]) -> Self {
		Self::Bytes(value)
	}
}
