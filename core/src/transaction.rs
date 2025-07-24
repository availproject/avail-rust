use super::{AccountId, MultiAddress, MultiSignature};
use codec::{Compact, Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use subxt_core::config::{Hasher, substrate::BlakeTwo256};
use subxt_signer::sr25519::Keypair;

pub use subxt_core::utils::Era;

/// Current version of the [`UncheckedExtrinsic`] encoded format.
///
/// This version needs to be bumped if the encoded representation changes.
/// It ensures that if the representation is changed and the format is not known,
/// the decoding fails.
pub const EXTRINSIC_FORMAT_VERSION: u8 = 4;

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct TransactionExtra {
	pub era: Era,
	#[codec(compact)]
	pub nonce: u32,
	#[codec(compact)]
	pub tip: u128,
	#[codec(compact)]
	pub app_id: u32,
}

#[derive(Debug, Clone)]
pub struct TransactionAdditional {
	pub spec_version: u32,
	pub tx_version: u32,
	pub genesis_hash: H256,
	pub fork_hash: H256,
}
impl Encode for TransactionAdditional {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.spec_version.encode_to(dest);
		self.tx_version.encode_to(dest);
		self.genesis_hash.encode_to(dest);
		self.fork_hash.encode_to(dest);
	}
}
impl Decode for TransactionAdditional {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let spec_version = Decode::decode(input)?;
		let tx_version = Decode::decode(input)?;
		let genesis_hash = Decode::decode(input)?;
		let fork_hash = Decode::decode(input)?;
		Ok(Self { spec_version, tx_version, genesis_hash, fork_hash })
	}
}

#[derive(Debug, Clone)]
pub struct AlreadyEncoded(pub Vec<u8>);
impl Encode for AlreadyEncoded {
	fn size_hint(&self) -> usize {
		self.0.len()
	}

	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		dest.write(&self.0);
	}
}
impl Decode for AlreadyEncoded {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let length = input.remaining_len()?;
		let Some(length) = length else {
			return Err("Failed to decode transaction".into());
		};
		if length == 0 {
			return Ok(Self(Vec::new()));
		}
		let mut value = vec![0u8; length];
		input.read(&mut value)?;
		Ok(Self(value))
	}
}

impl From<Vec<u8>> for AlreadyEncoded {
	fn from(value: Vec<u8>) -> Self {
		AlreadyEncoded(value)
	}
}

#[derive(Debug, Clone)]
pub struct TransactionCall {
	pub pallet_id: u8,
	pub call_id: u8,
	pub data: AlreadyEncoded,
}
impl TransactionCall {
	pub fn new(pallet_id: u8, call_id: u8, data: Vec<u8>) -> Self {
		Self { pallet_id, call_id, data: AlreadyEncoded::from(data) }
	}
}
impl Encode for TransactionCall {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.pallet_id.encode_to(dest);
		self.call_id.encode_to(dest);
		self.data.encode_to(dest);
	}
}
impl Decode for TransactionCall {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let pallet_id = Decode::decode(input)?;
		let call_id = Decode::decode(input)?;
		let data = Decode::decode(input)?;
		Ok(Self { pallet_id, call_id, data })
	}
}

// There is no need for Encode and Decode
#[derive(Debug, Clone)]
pub struct TransactionPayload<'a> {
	pub call: Cow<'a, TransactionCall>,
	pub extra: TransactionExtra,
	pub additional: TransactionAdditional,
}

impl<'a> TransactionPayload<'a> {
	pub fn new(call: TransactionCall, extra: TransactionExtra, additional: TransactionAdditional) -> Self {
		Self { call: Cow::Owned(call), extra, additional }
	}

	pub fn new_borrowed(call: &'a TransactionCall, extra: TransactionExtra, additional: TransactionAdditional) -> Self {
		Self { call: Cow::Borrowed(call), extra, additional }
	}

	pub fn sign(&self, signer: &Keypair) -> [u8; 64] {
		let call = self.call.as_ref();
		let size_hint = call.size_hint() + self.extra.size_hint() + self.additional.size_hint();

		let mut data: Vec<u8> = Vec::with_capacity(size_hint);
		self.call.encode_to(&mut data);
		self.extra.encode_to(&mut data);
		self.additional.encode_to(&mut data);

		if data.len() > 256 {
			let hash = BlakeTwo256::hash(&data);
			signer.sign(hash.as_ref()).0
		} else {
			signer.sign(&data).0
		}
	}
}

#[derive(Debug, Clone)]
pub struct TransactionSigned {
	pub address: MultiAddress,
	pub signature: MultiSignature,
	pub tx_extra: TransactionExtra,
}
impl Encode for TransactionSigned {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.address.encode_to(dest);
		self.signature.encode_to(dest);
		self.tx_extra.encode_to(dest);
	}
}
impl Decode for TransactionSigned {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let address = Decode::decode(input)?;
		let signature = Decode::decode(input)?;
		let tx_extra = Decode::decode(input)?;
		Ok(Self { address, signature, tx_extra })
	}
}

#[derive(Debug, Clone)]
pub struct Transaction<'a> {
	pub signed: Option<TransactionSigned>,
	pub call: Cow<'a, TransactionCall>,
}

impl<'a> Transaction<'a> {
	pub fn new(account_id: AccountId, signature: [u8; 64], payload: TransactionPayload<'a>) -> Self {
		let address = MultiAddress::Id(account_id);
		let signature = MultiSignature::Sr25519(signature);

		let signed = Some(TransactionSigned { address, signature, tx_extra: payload.extra.clone() });

		Self { signed, call: payload.call }
	}

	pub fn encode(&self) -> Vec<u8> {
		let mut encoded_tx_inner = Vec::new();
		if let Some(signed) = &self.signed {
			0x84u8.encode_to(&mut encoded_tx_inner);
			signed.address.encode_to(&mut encoded_tx_inner);
			signed.signature.encode_to(&mut encoded_tx_inner);
			signed.tx_extra.encode_to(&mut encoded_tx_inner);
		} else {
			0x4u8.encode_to(&mut encoded_tx_inner);
		}

		let call = self.call.as_ref();
		call.encode_to(&mut encoded_tx_inner);
		let mut encoded_tx = Compact(encoded_tx_inner.len() as u32).encode();
		encoded_tx.append(&mut encoded_tx_inner);

		encoded_tx
	}

	pub fn hash(&self) -> H256 {
		let encoded = self.encode();
		BlakeTwo256::hash(&encoded)
	}
}

impl<'a> TryFrom<&Vec<u8>> for Transaction<'a> {
	type Error = codec::Error;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl<'a> TryFrom<&[u8]> for Transaction<'a> {
	type Error = codec::Error;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let mut value = value;
		Self::decode(&mut value)
	}
}

impl<'a> Decode for Transaction<'a> {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		// This is a little more complicated than usual since the binary format must be compatible
		// with SCALE's generic `Vec<u8>` type. Basically this just means accepting that there
		// will be a prefix of vector length.
		let expected_length = Compact::<u32>::decode(input)?;
		let before_length = input.remaining_len()?;

		let version = input.read_byte()?;

		let is_signed = version & 0b1000_0000 != 0;
		let version = version & 0b0111_1111;
		if version != EXTRINSIC_FORMAT_VERSION {
			return Err("Invalid transaction version".into());
		}

		let signed = is_signed.then(|| TransactionSigned::decode(input)).transpose()?;
		let call = TransactionCall::decode(input)?;

		if let Some((before_length, after_length)) = input.remaining_len()?.and_then(|a| before_length.map(|b| (b, a)))
		{
			let length = before_length.saturating_sub(after_length);

			if length != expected_length.0 as usize {
				return Err("Invalid length prefix".into());
			}
		}

		Ok(Self { signed, call: Cow::Owned(call) })
	}
}

impl<'a> Serialize for Transaction<'a> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let bytes = self.encode();
		impl_serde::serialize::serialize(&bytes, serializer)
	}
}

impl<'a, 'b> Deserialize<'a> for Transaction<'b> {
	fn deserialize<D>(de: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let r = impl_serde::serialize::deserialize(de)?;
		Decode::decode(&mut &r[..]).map_err(|e| serde::de::Error::custom(format!("Decode error: {}", e)))
	}
}
