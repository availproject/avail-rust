use crate::{
	HasHeader,
	types::{AccountId, ExtrinsicExtra, ExtrinsicSignature, H256, MultiAddress, MultiSignature},
};
use codec::{Compact, Decode, Encode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use subxt_core::config::{Hasher, substrate::BlakeTwo256};
use subxt_signer::sr25519::Keypair;

/// Current version of the [`UncheckedExtrinsic`] encoded format.
///
/// This version needs to be bumped if the encoded representation changes.
/// It ensures that if the representation is changed and the format is not known,
/// the decoding fails.
pub const EXTRINSIC_FORMAT_VERSION: u8 = 4;

#[derive(Debug, Clone)]
pub struct ExtrinsicAdditional {
	pub spec_version: u32,
	pub tx_version: u32,
	pub genesis_hash: H256,
	pub fork_hash: H256,
}
impl Encode for ExtrinsicAdditional {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.spec_version.encode_to(dest);
		self.tx_version.encode_to(dest);
		self.genesis_hash.encode_to(dest);
		self.fork_hash.encode_to(dest);
	}
}
impl Decode for ExtrinsicAdditional {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let spec_version = Decode::decode(input)?;
		let tx_version = Decode::decode(input)?;
		let genesis_hash = Decode::decode(input)?;
		let fork_hash = Decode::decode(input)?;
		Ok(Self { spec_version, tx_version, genesis_hash, fork_hash })
	}
}

#[derive(Debug, Clone)]
pub struct ExtrinsicCall {
	pub pallet_id: u8,
	pub variant_id: u8,
	pub data: Vec<u8>,
}
impl ExtrinsicCall {
	pub fn new(pallet_id: u8, variant_id: u8, data: Vec<u8>) -> Self {
		Self { pallet_id, variant_id, data }
	}
}
impl Encode for ExtrinsicCall {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.pallet_id.encode_to(dest);
		self.variant_id.encode_to(dest);
		dest.write(&self.data);
	}
}
impl Decode for ExtrinsicCall {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let pallet_id = Decode::decode(input)?;
		let variant_id = Decode::decode(input)?;
		let data = crate::utils::decode_already_decoded(input)?;
		Ok(Self { pallet_id, variant_id, data })
	}
}

impl<T: HasHeader + Encode> From<&T> for ExtrinsicCall {
	fn from(value: &T) -> Self {
		Self {
			pallet_id: T::HEADER_INDEX.0,
			variant_id: T::HEADER_INDEX.1,
			data: value.encode(),
		}
	}
}

// There is no need for Encode and Decode
#[derive(Debug, Clone)]
pub struct ExtrinsicPayload<'a> {
	pub call: Cow<'a, ExtrinsicCall>,
	pub extra: ExtrinsicExtra,
	pub additional: ExtrinsicAdditional,
}

impl<'a> ExtrinsicPayload<'a> {
	pub fn new(call: ExtrinsicCall, extra: ExtrinsicExtra, additional: ExtrinsicAdditional) -> Self {
		Self { call: Cow::Owned(call), extra, additional }
	}

	pub fn new_borrowed(call: &'a ExtrinsicCall, extra: ExtrinsicExtra, additional: ExtrinsicAdditional) -> Self {
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
pub struct GenericExtrinsic<'a> {
	pub signature: Option<ExtrinsicSignature>,
	pub call: Cow<'a, ExtrinsicCall>,
}

impl<'a> GenericExtrinsic<'a> {
	pub fn new(account_id: AccountId, signature: [u8; 64], payload: ExtrinsicPayload<'a>) -> Self {
		let address = MultiAddress::Id(account_id);
		let signature = MultiSignature::Sr25519(signature);
		let signature = Some(ExtrinsicSignature { address, signature, tx_extra: payload.extra.clone() });

		Self { signature, call: payload.call }
	}

	pub fn encode(&self) -> Vec<u8> {
		let mut encoded_tx_inner = Vec::new();
		if let Some(signed) = &self.signature {
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

impl TryFrom<&Vec<u8>> for GenericExtrinsic<'_> {
	type Error = codec::Error;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&[u8]> for GenericExtrinsic<'_> {
	type Error = codec::Error;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let mut value = value;
		Self::decode(&mut value)
	}
}

impl Decode for GenericExtrinsic<'_> {
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

		let signed = is_signed.then(|| ExtrinsicSignature::decode(input)).transpose()?;
		let call = ExtrinsicCall::decode(input)?;

		if let Some((before_length, after_length)) = input.remaining_len()?.and_then(|a| before_length.map(|b| (b, a)))
		{
			let length = before_length.saturating_sub(after_length);

			if length != expected_length.0 as usize {
				return Err("Invalid length prefix".into());
			}
		}

		Ok(Self { signature: signed, call: Cow::Owned(call) })
	}
}

impl Serialize for GenericExtrinsic<'_> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let bytes = self.encode();
		impl_serde::serialize::serialize(&bytes, serializer)
	}
}

impl<'a> Deserialize<'a> for GenericExtrinsic<'_> {
	fn deserialize<D>(de: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let r = impl_serde::serialize::deserialize(de)?;
		Decode::decode(&mut &r[..]).map_err(|e| serde::de::Error::custom(format!("Decode error: {}", e)))
	}
}
