use super::transaction::{AlreadyEncoded, TransactionSigned, EXTRINSIC_FORMAT_VERSION};
use codec::{Compact, Decode, Encode, Error, Input};
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "generated_metadata"))]
use crate::avail::{RuntimeCall, RuntimeEvent};
#[cfg(feature = "generated_metadata")]
use crate::avail_generated::runtime_types::da_runtime::{RuntimeCall, RuntimeEvent};

#[derive(Clone)]
pub struct OpaqueTransaction {
	/// The signature, address, number of extrinsics have come before from
	/// the same signer and an era describing the longevity of this transaction,
	/// if this is a signed extrinsic.
	pub signature: Option<TransactionSigned>,
	/// The function that should be called.
	pub call: Vec<u8>,
}

impl OpaqueTransaction {
	pub fn pallet_index(&self) -> u8 {
		self.call[0]
	}

	pub fn call_index(&self) -> u8 {
		self.call[1]
	}
}

impl TryFrom<Vec<u8>> for OpaqueTransaction {
	type Error = codec::Error;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&Vec<u8>> for OpaqueTransaction {
	type Error = codec::Error;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&[u8]> for OpaqueTransaction {
	type Error = codec::Error;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let mut value = value;
		Self::decode(&mut value)
	}
}

impl Decode for OpaqueTransaction {
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
		let call: AlreadyEncoded = Decode::decode(input)?;

		if let Some((before_length, after_length)) = input.remaining_len()?.and_then(|a| before_length.map(|b| (b, a)))
		{
			let length = before_length.saturating_sub(after_length);

			if length != expected_length.0 as usize {
				return Err("Invalid length prefix".into());
			}
		}

		Ok(Self {
			signature,
			call: call.0,
		})
	}
}

impl Encode for OpaqueTransaction {
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

impl Serialize for OpaqueTransaction {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let bytes = self.encode();
		impl_serde::serialize::serialize(&bytes, serializer)
	}
}

impl<'a> Deserialize<'a> for OpaqueTransaction {
	fn deserialize<D>(de: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let r = impl_serde::serialize::deserialize(de)?;
		Decode::decode(&mut &r[..]).map_err(|e| serde::de::Error::custom(format!("Decode error: {}", e)))
	}
}

#[derive(Clone)]
pub struct DecodedTransaction {
	/// The signature, address, number of extrinsics have come before from
	/// the same signer and an era describing the longevity of this transaction,
	/// if this is a signed extrinsic.
	pub signature: Option<TransactionSigned>,
	/// The function that should be called.
	pub call: RuntimeCall,
}

impl DecodedTransaction {
	pub fn app_id(&self) -> Option<u32> {
		self.signature.as_ref().map(|s| s.tx_extra.app_id)
	}

	#[cfg(not(feature = "generated_metadata"))]
	pub fn pallet_index(&self) -> u8 {
		self.call.pallet_index()
	}

	#[cfg(not(feature = "generated_metadata"))]
	pub fn call_index(&self) -> u8 {
		self.call.call_index()
	}
}

impl TryFrom<&Vec<u8>> for DecodedTransaction {
	type Error = codec::Error;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&[u8]> for DecodedTransaction {
	type Error = codec::Error;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let mut value = value;
		Self::decode(&mut value)
	}
}

impl Decode for DecodedTransaction {
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
		let call = Decode::decode(input)?;

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

impl Encode for DecodedTransaction {
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

		self.call.encode_to(&mut encoded_tx_inner);
		let mut encoded_tx = Compact(encoded_tx_inner.len() as u32).encode();
		encoded_tx.append(&mut encoded_tx_inner);

		dest.write(&encoded_tx)
	}
}

impl Serialize for DecodedTransaction {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let bytes = self.encode();
		impl_serde::serialize::serialize(&bytes, serializer)
	}
}

impl<'a> Deserialize<'a> for DecodedTransaction {
	fn deserialize<D>(de: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let r = impl_serde::serialize::deserialize(de)?;
		Decode::decode(&mut &r[..]).map_err(|e| serde::de::Error::custom(format!("Decode error: {}", e)))
	}
}

#[derive(Debug, Clone, Decode)]
pub struct DecodedEvent {
	pub phase: RuntimePhase,
	pub event: RuntimeEvent,
}

/// A phase of a block's execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum RuntimePhase {
	/// Applying an extrinsic.
	ApplyExtrinsic(u32),
	/// Finalizing the block.
	Finalization,
	/// Initializing the block.
	Initialization,
}

#[cfg(test)]
pub mod test {
	use std::borrow::Cow;

	use codec::Encode;
	use subxt_core::utils::AccountId32;
	use subxt_core::utils::Era;

	use crate::avail::data_availability::tx::SubmitData;
	use crate::decoded_transaction::OpaqueTransaction;
	use crate::transaction::TransactionSigned;
	use crate::DecodedTransaction;
	use crate::Transaction;
	use crate::{chain_types::TransactionCallLike, TransactionExtra};
	use crate::{MultiAddress, MultiSignature};

	#[test]
	fn test_encoding_decoding() {
		let call = SubmitData { data: vec![0, 1, 2, 3] }.to_call();

		let account_id = AccountId32([1u8; 32]);
		let signature = [1u8; 64];
		let signed = TransactionSigned {
			address: MultiAddress::Id(account_id),
			signature: MultiSignature::Sr25519(signature),
			tx_extra: TransactionExtra {
				era: Era::Mortal { period: 4, phase: 2 },
				nonce: 1,
				tip: 2u128,
				app_id: 3,
			},
		};

		let tx = Transaction {
			signed: Some(signed.clone()),
			call: Cow::Owned(call.clone()),
		};

		let encoded_tx = tx.encode();

		// Opaque Transaction
		let opaque = OpaqueTransaction::try_from(&encoded_tx).unwrap();
		let opaque_encoded = opaque.encode();

		assert_eq!(encoded_tx, opaque_encoded);

		// Decoded Transaction
		let decoded = DecodedTransaction::try_from(&encoded_tx).unwrap();
		let decoded_encoded = decoded.encode();

		assert_eq!(encoded_tx, decoded_encoded);
	}

	#[test]
	fn test_serialize_deserialize() {
		let call = SubmitData { data: vec![0, 1, 2, 3] }.to_call();

		let account_id = AccountId32([1u8; 32]);
		let signature = [1u8; 64];
		let signed = TransactionSigned {
			address: MultiAddress::Id(account_id),
			signature: MultiSignature::Sr25519(signature),
			tx_extra: TransactionExtra {
				era: Era::Mortal { period: 4, phase: 2 },
				nonce: 1,
				tip: 2u128,
				app_id: 3,
			},
		};

		let tx = Transaction {
			signed: Some(signed.clone()),
			call: Cow::Owned(call.clone()),
		};

		let encoded_tx = tx.encode();
		let expected_serialized = std::format!("0x{}", hex::encode(&encoded_tx));

		// Transaction Serialized
		let serialized = serde_json::to_string(&tx).unwrap();
		assert_eq!(serialized.trim_matches('"'), expected_serialized);

		// Transaction Deserialized
		let tx_deserialized: Transaction = serde_json::from_str(&serialized).unwrap();
		assert_eq!(encoded_tx, tx_deserialized.encode());

		// Opaque Serialized
		let opaque = OpaqueTransaction::try_from(&encoded_tx).unwrap();
		let serialized = serde_json::to_string(&opaque).unwrap();
		assert_eq!(serialized.trim_matches('"'), expected_serialized);

		// Opaque Deserialized
		let opaque_deserialized: OpaqueTransaction = serde_json::from_str(&serialized).unwrap();
		assert_eq!(encoded_tx, opaque_deserialized.encode());

		// Decoded Serialized
		let decoded = DecodedTransaction::try_from(&encoded_tx).unwrap();
		let serialized = serde_json::to_string(&decoded).unwrap();
		assert_eq!(serialized.trim_matches('"'), expected_serialized);

		// Decoded Deserialized
		let decoded_deserialized: DecodedTransaction = serde_json::from_str(&serialized).unwrap();
		assert_eq!(encoded_tx, decoded_deserialized.encode());
	}
}
