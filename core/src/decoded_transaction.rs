use super::transaction::{AlreadyEncoded, EXTRINSIC_FORMAT_VERSION, TransactionSigned};
use crate::TransactionCall;
use codec::{Compact, Decode, Encode, Error, Input};
use serde::{Deserialize, Serialize};

pub trait HasTxDispatchIndex {
	// Pallet ID, Call ID
	const DISPATCH_INDEX: (u8, u8);
}

pub trait TransactionCallLike {
	fn to_call(&self) -> TransactionCall;
	fn decode_call(call: &[u8]) -> Option<Box<Self>>;
	fn decode_call_data(call_data: &[u8]) -> Option<Box<Self>>;
}

impl<T: HasTxDispatchIndex + Encode + Decode> TransactionCallLike for T {
	fn to_call(&self) -> TransactionCall {
		TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, self.encode())
	}

	fn decode_call(call: &[u8]) -> Option<Box<T>> {
		// This was moved out in order to decrease compilation times
		if !tx_filter_in(call, Self::DISPATCH_INDEX) {
			return None;
		}

		if call.len() <= 2 {
			try_decode_transaction(&[])
		} else {
			try_decode_transaction(&call[2..])
		}
	}

	fn decode_call_data(call_data: &[u8]) -> Option<Box<Self>> {
		// This was moved out in order to decrease compilation times
		try_decode_transaction(call_data)
	}
}

// Purely here to decrease compilation times
#[inline(never)]
fn try_decode_transaction<T: Decode>(mut event_data: &[u8]) -> Option<Box<T>> {
	T::decode(&mut event_data).ok().map(Box::new)
}

// Purely here to decrease compilation times
fn tx_filter_in(call: &[u8], dispatch_index: (u8, u8)) -> bool {
	if call.len() < 3 {
		return false;
	}

	let (pallet_id, call_id) = (call[0], call[1]);
	if dispatch_index.0 != pallet_id || dispatch_index.1 != call_id {
		return false;
	}

	true
}

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

#[cfg(test)]
pub mod test {
	use super::TransactionCallLike;
	use std::borrow::Cow;

	use codec::Encode;
	use subxt_core::utils::{AccountId32, Era};

	use crate::{
		MultiAddress, MultiSignature, Transaction, TransactionExtra, avail::data_availability::tx::SubmitData,
		decoded_transaction::OpaqueTransaction, transaction::TransactionSigned,
	};

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
	}
}
