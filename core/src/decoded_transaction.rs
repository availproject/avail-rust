use super::substrate::extrinsic::EXTRINSIC_FORMAT_VERSION;
use crate::{
	types::{ExtrinsicSignature, metadata::StringOrBytes},
	utils::decode_already_decoded,
};
use codec::{Compact, Decode, Encode, Error, Input};
use serde::{Deserialize, Serialize};

pub trait HasHeader {
	// Pallet ID, Variant ID
	const HEADER_INDEX: (u8, u8);
}

pub trait TransactionDecodable: Sized {
	fn decode_call<'a>(call: impl Into<StringOrBytes<'a>>) -> Result<Self, String>;
}

impl<T: HasHeader + Decode> TransactionDecodable for T {
	fn decode_call<'a>(call: impl Into<StringOrBytes<'a>>) -> Result<T, String> {
		fn inner<T: HasHeader + Decode>(call: StringOrBytes) -> Result<T, String> {
			let call = match call {
				StringOrBytes::String(s) => {
					&const_hex::decode(s.trim_start_matches("0x")).map_err(|x| x.to_string())?
				},
				StringOrBytes::Bytes(b) => b,
			};

			if !tx_filter_in(call, T::HEADER_INDEX) {
				return Err("Failed to decode extrinsic. TODO".into());
			}

			if call.len() <= 2 {
				let mut data: &[u8] = &[];
				Ok(T::decode(&mut data).map_err(|x| x.to_string())?)
			} else {
				let mut data = &call[2..];
				Ok(T::decode(&mut data).map_err(|x| x.to_string())?)
			}
		}

		inner(call.into())
	}
}

#[inline(never)]
fn tx_filter_in(call: &[u8], header: (u8, u8)) -> bool {
	if call.len() < 3 {
		return false;
	}

	let (pallet_id, variant_id) = (call[0], call[1]);
	if header.0 != pallet_id || header.1 != variant_id {
		return false;
	}

	true
}

#[derive(Clone)]
pub struct RawExtrinsic {
	/// The signature, address, number of extrinsics have come before from
	/// the same signer and an era describing the longevity of this transaction,
	/// if this is a signed extrinsic.
	pub signature: Option<ExtrinsicSignature>,
	/// The function that should be called.
	pub call: Vec<u8>,
}

impl<'a> TryFrom<StringOrBytes<'a>> for RawExtrinsic {
	type Error = String;

	fn try_from(value: StringOrBytes<'a>) -> Result<Self, Self::Error> {
		match value {
			StringOrBytes::String(s) => Self::try_from(s),
			StringOrBytes::Bytes(b) => Self::try_from(b),
		}
	}
}

impl TryFrom<Vec<u8>> for RawExtrinsic {
	type Error = String;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&Vec<u8>> for RawExtrinsic {
	type Error = String;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&[u8]> for RawExtrinsic {
	type Error = String;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let mut value = value;
		Self::decode(&mut value).map_err(|x| x.to_string())
	}
}

impl TryFrom<String> for RawExtrinsic {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl TryFrom<&str> for RawExtrinsic {
	type Error = String;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let Ok(hex_decoded) = const_hex::decode(value.trim_start_matches("0x")) else {
			return Err("Failed to hex decode transaction".into());
		};

		Self::try_from(hex_decoded.as_slice())
	}
}

impl Decode for RawExtrinsic {
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		// This is a little more complicated than usual since the binary format must be compatible
		// with SCALE's generic `Vec<u8>` type. Basically this just means accepting that there
		// will be a prefix of vector length.
		let expected_length: Compact<u32> = Decode::decode(input)?;
		let before_length = input.remaining_len()?;

		let version = input.read_byte()?;

		let is_signed = version & 0b1000_0000 != 0;
		let version = version & 0b0111_1111;
		if version != EXTRINSIC_FORMAT_VERSION {
			return Err("Invalid transaction version".into());
		}

		let signature = is_signed.then(|| Decode::decode(input)).transpose()?;
		let call = decode_already_decoded(input)?;

		if let Some((before_length, after_length)) = input.remaining_len()?.and_then(|a| before_length.map(|b| (b, a)))
		{
			let length = before_length.saturating_sub(after_length);

			if length != expected_length.0 as usize {
				return Err("Invalid length prefix".into());
			}
		}

		Ok(Self { signature, call })
	}
}

impl Encode for RawExtrinsic {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		let mut encoded_tx_inner = Vec::new();
		if let Some(signed) = &self.signature {
			0x84u8.encode_to(&mut encoded_tx_inner);
			signed.address.encode_to(&mut encoded_tx_inner);
			signed.signature.encode_to(&mut encoded_tx_inner);
			signed.tx_extra.encode_to(&mut encoded_tx_inner);
		} else {
			0x4u8.encode_to(&mut encoded_tx_inner);
		}

		encoded_tx_inner.extend(&self.call);
		let mut encoded_tx = Compact(encoded_tx_inner.len() as u32).encode();
		encoded_tx.append(&mut encoded_tx_inner);

		dest.write(&encoded_tx)
	}
}

impl Serialize for RawExtrinsic {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let bytes = self.encode();
		impl_serde::serialize::serialize(&bytes, serializer)
	}
}

impl<'a> Deserialize<'a> for RawExtrinsic {
	fn deserialize<D>(de: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let r = impl_serde::serialize::deserialize(de)?;
		Decode::decode(&mut &r[..]).map_err(|e| serde::de::Error::custom(format!("Decode error: {}", e)))
	}
}

#[derive(Debug, Clone)]
pub struct SignedExtrinsic<T: HasHeader + Decode + Sized> {
	pub signature: ExtrinsicSignature,
	pub call: T,
}

impl<'a, T: HasHeader + Decode> TryFrom<StringOrBytes<'a>> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: StringOrBytes<'a>) -> Result<Self, Self::Error> {
		match value {
			StringOrBytes::String(s) => Self::try_from(s),
			StringOrBytes::Bytes(b) => Self::try_from(b),
		}
	}
}

impl<T: HasHeader + Decode> TryFrom<Vec<u8>> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl<T: HasHeader + Decode> TryFrom<&Vec<u8>> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl<T: HasHeader + Decode> TryFrom<&[u8]> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let ext = RawExtrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}

impl<T: HasHeader + Decode> TryFrom<String> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl<T: HasHeader + Decode> TryFrom<&String> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: &String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl<T: HasHeader + Decode> TryFrom<&str> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let ext = RawExtrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}

impl<T: HasHeader + Decode> TryFrom<RawExtrinsic> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: RawExtrinsic) -> Result<Self, Self::Error> {
		let signature = value.signature.ok_or("Extrinsic has no signature")?;
		let call = T::decode_call(&value.call)?;
		Ok(Self { signature, call })
	}
}

impl<T: HasHeader + Decode> TryFrom<&RawExtrinsic> for SignedExtrinsic<T> {
	type Error = String;

	fn try_from(value: &RawExtrinsic) -> Result<Self, Self::Error> {
		let signature = value.signature.as_ref().ok_or("Extrinsic has no signature")?.clone();
		let call = T::decode_call(&value.call)?;
		Ok(Self { signature, call })
	}
}

#[derive(Debug, Clone)]
pub struct Extrinsic<T: HasHeader + Decode + Sized> {
	pub signature: Option<ExtrinsicSignature>,
	pub call: T,
}

impl<'a, T: HasHeader + Decode> TryFrom<StringOrBytes<'a>> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: StringOrBytes<'a>) -> Result<Self, Self::Error> {
		match value {
			StringOrBytes::String(s) => Self::try_from(s),
			StringOrBytes::Bytes(b) => Self::try_from(b),
		}
	}
}

impl<T: HasHeader + Decode> TryFrom<Vec<u8>> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl<T: HasHeader + Decode> TryFrom<&Vec<u8>> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl<T: HasHeader + Decode> TryFrom<&[u8]> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let ext = RawExtrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}

impl<T: HasHeader + Decode> TryFrom<String> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl<T: HasHeader + Decode> TryFrom<&String> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: &String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl<T: HasHeader + Decode> TryFrom<&str> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let ext = RawExtrinsic::try_from(value)?;
		Self::try_from(ext)
	}
}

impl<T: HasHeader + Decode> TryFrom<RawExtrinsic> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: RawExtrinsic) -> Result<Self, Self::Error> {
		Self::try_from(&value)
	}
}

impl<T: HasHeader + Decode> TryFrom<&RawExtrinsic> for Extrinsic<T> {
	type Error = String;

	fn try_from(value: &RawExtrinsic) -> Result<Self, Self::Error> {
		let call = T::decode_call(&value.call)?;
		Ok(Self { signature: value.signature.clone(), call })
	}
}

#[cfg(test)]
pub mod test {
	/* 	#[test]
	fn test_encoding_decoding() {
		let call = SubmitData { data: vec![0, 1, 2, 3] }.to_call();

		let account_id = AccountId32([1u8; 32]);
		let signature = [1u8; 64];
		let signed = SignedExtra {
			address: MultiAddress::Id(account_id),
			signature: MultiSignature::Sr25519(signature),
			tx_extra: ExtrinsicExtra {
				era: Era::Mortal { period: 4, phase: 2 },
				nonce: 1,
				tip: 2u128,
				app_id: 3,
			},
		};

		let tx = Extrinsic {
			signature: Some(signed.clone()),
			call: Cow::Owned(call.clone()),
		};

		let encoded_tx = tx.encode();

		// Opaque Transaction
		let opaque = RawExtrinsic::try_from(&encoded_tx).unwrap();
		let opaque_encoded = opaque.encode();

		assert_eq!(encoded_tx, opaque_encoded);
	}

	#[test]
	fn test_serialize_deserialize() {
		let call = SubmitData { data: vec![0, 1, 2, 3] }.to_call();

		let account_id = AccountId32([1u8; 32]);
		let signature = [1u8; 64];
		let signed = SignedExtra {
			address: MultiAddress::Id(account_id),
			signature: MultiSignature::Sr25519(signature),
			tx_extra: ExtrinsicExtra {
				era: Era::Mortal { period: 4, phase: 2 },
				nonce: 1,
				tip: 2u128,
				app_id: 3,
			},
		};

		let tx = Extrinsic {
			signature: Some(signed.clone()),
			call: Cow::Owned(call.clone()),
		};

		let encoded_tx = tx.encode();
		let expected_serialized = std::format!("0x{}", const_hex::encode(&encoded_tx));

		// Transaction Serialized
		let serialized = serde_json::to_string(&tx).unwrap();
		assert_eq!(serialized.trim_matches('"'), expected_serialized);

		// Transaction Deserialized
		let tx_deserialized: Extrinsic = serde_json::from_str(&serialized).unwrap();
		assert_eq!(encoded_tx, tx_deserialized.encode());

		// Opaque Serialized
		let opaque = RawExtrinsic::try_from(&encoded_tx).unwrap();
		let serialized = serde_json::to_string(&opaque).unwrap();
		assert_eq!(serialized.trim_matches('"'), expected_serialized);

		// Opaque Deserialized
		let opaque_deserialized: RawExtrinsic = serde_json::from_str(&serialized).unwrap();
		assert_eq!(encoded_tx, opaque_deserialized.encode());
	} */
}
