use codec::{Decode, Encode};
use primitive_types::H256;
use subxt_core::utils::AccountId32;

pub type AccountId = AccountId32;
pub type AccountIndex = u32;
pub type BlockHeight = u32;
pub type BlockHash = H256;
pub type Signature = MultiSignature;

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
