use codec::{Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
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

pub type DispatchIndex = (u8, u8);
pub type EmittedIndex = (u8, u8);

#[derive(Debug, Clone, Copy, Deserialize)]
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

#[derive(Debug, Clone, Copy, Deserialize)]
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
