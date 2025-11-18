use std::{fmt::Display, str::FromStr};

use codec::{Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Serialize};

use crate::{AccountId, MultiAddress, utils::account_id_from_str};

#[derive(Debug, Clone, Copy, Default, Encode, Decode, Eq, PartialEq)]
pub struct AppId(#[codec(compact)] pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct ChainInfo {
	pub best_hash: H256,
	pub best_height: u32,
	pub finalized_hash: H256,
	pub finalized_height: u32,
	pub genesis_hash: H256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct BlockInfo {
	pub hash: H256,
	pub height: u32,
}

impl std::fmt::Display for BlockInfo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Height: {}, Hash: {:?}", self.height, self.hash)
	}
}

impl From<(H256, u32)> for BlockInfo {
	fn from(value: (H256, u32)) -> Self {
		Self { hash: value.0, height: value.1 }
	}
}

impl From<BlockInfo> for H256 {
	fn from(value: BlockInfo) -> Self {
		value.hash
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HashNumber {
	Hash(H256),
	Number(u32),
}

impl From<BlockInfo> for HashNumber {
	fn from(value: BlockInfo) -> Self {
		Self::Hash(value.hash)
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
	type Error = String;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let h256 = H256::from_str(value).map_err(|e| e.to_string())?;
		Ok(Self::Hash(h256))
	}
}

impl TryFrom<String> for HashNumber {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		HashNumber::try_from(value.as_str())
	}
}

impl TryFrom<HashStringNumber> for HashNumber {
	type Error = String;

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
	type Error = String;

	fn try_from(value: HashString) -> Result<Self, Self::Error> {
		Self::try_from(HashStringNumber::from(value))
	}
}

#[derive(Debug, Clone)]
pub enum HashString {
	Hash(H256),
	String(String),
}

impl Display for HashString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			HashString::Hash(x) => write!(f, "{}", const_hex::encode_prefixed(x.0)),
			HashString::String(x) => write!(f, "{}", x),
		}
	}
}

impl TryInto<H256> for HashString {
	type Error = String;

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

impl std::fmt::Display for HashStringNumber {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			HashStringNumber::Hash(h) => write!(f, "Hash: {}", h),
			HashStringNumber::String(h) => write!(f, "String Hash: {}", h),
			HashStringNumber::Number(n) => write!(f, "Number: {}", n),
		}
	}
}

impl From<BlockInfo> for HashStringNumber {
	fn from(value: BlockInfo) -> Self {
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
	StringRef(&'a str),
	BoxedString(Box<str>),
	Bytes(&'a [u8]),
	BoxedBytes(Box<[u8]>),
}

impl<'a> From<StringOrBytes<'a>> for Vec<u8> {
	fn from(value: StringOrBytes<'a>) -> Self {
		match value {
			StringOrBytes::StringRef(x) => x.as_bytes().to_vec(),
			StringOrBytes::BoxedString(x) => x.as_bytes().to_vec(),
			StringOrBytes::Bytes(x) => x.to_vec(),
			StringOrBytes::BoxedBytes(x) => x.into_vec(),
		}
	}
}

impl<'a> From<String> for StringOrBytes<'a> {
	fn from(value: String) -> Self {
		Self::BoxedString(value.into())
	}
}

impl<'a> From<&'a String> for StringOrBytes<'a> {
	fn from(value: &'a String) -> Self {
		Self::StringRef(value.as_str())
	}
}

impl<'a> From<Vec<u8>> for StringOrBytes<'a> {
	fn from(value: Vec<u8>) -> Self {
		Self::BoxedBytes(value.into())
	}
}

impl<'a> From<&'a Vec<u8>> for StringOrBytes<'a> {
	fn from(value: &'a Vec<u8>) -> Self {
		Self::Bytes(value.as_slice())
	}
}

impl<'a> From<&'a str> for StringOrBytes<'a> {
	fn from(value: &'a str) -> Self {
		Self::StringRef(value)
	}
}

impl<'a> From<&'a [u8]> for StringOrBytes<'a> {
	fn from(value: &'a [u8]) -> Self {
		Self::Bytes(value)
	}
}

pub enum MultiAddressLike {
	MultiAddress(MultiAddress),
	BoxedString(Box<str>),
}

impl TryFrom<MultiAddressLike> for MultiAddress {
	type Error = String;

	fn try_from(value: MultiAddressLike) -> Result<Self, Self::Error> {
		match value {
			MultiAddressLike::MultiAddress(a) => Ok(a),
			MultiAddressLike::BoxedString(s) => account_id_from_str(&s).map(MultiAddress::from),
		}
	}
}

impl From<MultiAddress> for MultiAddressLike {
	fn from(value: MultiAddress) -> Self {
		Self::MultiAddress(value)
	}
}

impl From<AccountId> for MultiAddressLike {
	fn from(value: AccountId) -> Self {
		Self::MultiAddress(value.into())
	}
}

impl From<String> for MultiAddressLike {
	fn from(value: String) -> Self {
		Self::BoxedString(value.into())
	}
}

impl From<&String> for MultiAddressLike {
	fn from(value: &String) -> Self {
		Self::from(value.as_str())
	}
}

impl From<&str> for MultiAddressLike {
	fn from(value: &str) -> Self {
		Self::BoxedString(value.into())
	}
}

pub enum AccountIdLike {
	AccountId(AccountId),
	BoxedString(Box<str>),
}

impl TryFrom<AccountIdLike> for AccountId {
	type Error = String;

	fn try_from(value: AccountIdLike) -> Result<Self, Self::Error> {
		match value {
			AccountIdLike::AccountId(a) => Ok(a),
			AccountIdLike::BoxedString(s) => account_id_from_str(&s),
		}
	}
}

impl From<AccountId> for AccountIdLike {
	fn from(value: AccountId) -> Self {
		Self::AccountId(value)
	}
}

impl From<String> for AccountIdLike {
	fn from(value: String) -> Self {
		Self::BoxedString(value.into())
	}
}

impl From<&String> for AccountIdLike {
	fn from(value: &String) -> Self {
		Self::from(value.as_str())
	}
}

impl From<&str> for AccountIdLike {
	fn from(value: &str) -> Self {
		Self::BoxedString(value.into())
	}
}
