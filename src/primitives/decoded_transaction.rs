use super::transaction::{TransactionSigned, EXTRINSIC_FORMAT_VERSION};
use codec::{Compact, Decode, Encode, Error, Input};
use serde::{Deserialize, Serialize};

#[cfg(feature = "subxt_metadata")]
use crate::subxt_avail::runtime_types::da_runtime::{RuntimeCall, RuntimeEvent};
#[cfg(not(feature = "subxt_metadata"))]
pub type RuntimeCall = crate::avail::RuntimeCall;
#[cfg(not(feature = "subxt_metadata"))]
pub type RuntimeEvent = Vec<u8>;

#[derive(Clone)]
pub struct DecodedTransaction {
	/// The signature, address, number of extrinsics have come before from
	/// the same signer and an era describing the longevity of this transaction,
	/// if this is a signed extrinsic.
	pub signature: Option<TransactionSigned>,
	/// The function that should be called.
	pub function: RuntimeCall,
}

impl DecodedTransaction {
	/* 	fn encode_vec_compatible(inner: &[u8]) -> Vec<u8> {
		let compact_len = codec::Compact::<u32>(inner.len() as u32);

		// Allocate the output buffer with the correct length
		let mut output = Vec::with_capacity(compact_len.size_hint() + inner.len());

		compact_len.encode_to(&mut output);
		output.extend(inner);

		output
	} */

	pub fn app_id(&self) -> Option<u32> {
		self.signature.as_ref().map(|s| s.tx_extra.app_id)
	}
}

/* impl Encode for DecodedTransaction {
	fn encode(&self) -> Vec<u8> {
		let mut tmp = Vec::with_capacity(size_of::<Self>());

		// 1 byte version id.
		match self.signature.as_ref() {
			Some(s) => {
				tmp.push(EXTRINSIC_FORMAT_VERSION | 0b1000_0000);
				s.encode_to(&mut tmp);
			},
			None => {
				tmp.push(EXTRINSIC_FORMAT_VERSION & 0b0111_1111);
			},
		}
		self.function.encode_to(&mut tmp);
		Self::encode_vec_compatible(&tmp)
	}
}

impl EncodeLike for DecodedTransaction {} */

/* impl Serialize for DecodedTransaction {
	fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
	where
		S: ::serde::Serializer,
	{
		let encoded = self.encode();
		impl_serde::serialize::serialize(&encoded, s)
	}
} */

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
		let function = Decode::decode(input)?;

		if let Some((before_length, after_length)) = input.remaining_len()?.and_then(|a| before_length.map(|b| (b, a)))
		{
			let length = before_length.saturating_sub(after_length);

			if length != expected_length.0 as usize {
				return Err("Invalid length prefix".into());
			}
		}

		Ok(Self { signature, function })
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

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct UncheckedEvent {
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
