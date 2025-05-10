use std::borrow::Cow;

use codec::{Compact, Decode, Encode};
use primitive_types::H256;
use subxt_core::config::{substrate::BlakeTwo256, Hasher};
use subxt_core::utils::AccountId32;
use subxt_signer::sr25519::Keypair;

#[derive(Clone)]
pub struct TransactionCall {
	pub pallet_id: u8,
	pub call_id: u8,
	pub data: Vec<u8>,
}

impl TransactionCall {
	pub fn new(pallet_id: u8, call_id: u8, data: Vec<u8>) -> Self {
		Self {
			pallet_id,
			call_id,
			data,
		}
	}
}

impl Encode for TransactionCall {
	fn encode(&self) -> Vec<u8> {
		let mut dest = Vec::new();
		self.pallet_id.encode_to(&mut dest);
		self.call_id.encode_to(&mut dest);
		dest.append(&mut self.data.clone());

		dest
	}
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TransactionExtra {
	pub era: Era,
	#[codec(compact)]
	pub nonce: u32,
	#[codec(compact)]
	pub tip: u128,
	#[codec(compact)]
	pub app_id: u32,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TransactionAdditional {
	pub spec_version: u32,
	pub tx_version: u32,
	pub genesis_hash: H256,
	pub fork_hash: H256,
}

#[derive(Debug, Clone)]
pub enum Era {
	Immortal,
	Mortal { period: u64, phase: u64 },
}
impl Era {
	/// Create a new era based on a period (which should be a power of two between 4 and 65536
	/// inclusive) and a block number on which it should start (or, for long periods, be shortly
	/// after the start).
	///
	/// If using `Era` in the context of `FRAME` runtime, make sure that `period`
	/// does not exceed `BlockHashCount` parameter passed to `system` module, since that
	/// prunes old blocks and renders transactions immediately invalid.
	pub fn mortal(period: u64, block_number: u64) -> Self {
		let period = period.checked_next_power_of_two().unwrap_or(1 << 16).clamp(4, 1 << 16);
		let phase = block_number % period;
		let quantize_factor = (period >> 12).max(1);
		let quantized_phase = phase / quantize_factor * quantize_factor;

		Self::Mortal {
			period,
			phase: quantized_phase,
		}
	}

	pub fn immortal() -> Self {
		Self::Immortal
	}
}
impl Encode for Era {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		match self {
			Self::Immortal => dest.push_byte(0),
			Self::Mortal { period, phase } => {
				let quantize_factor = (*period >> 12).max(1);
				let encoded =
					(period.trailing_zeros() - 1).clamp(1, 15) as u16 | ((phase / quantize_factor) << 4) as u16;
				encoded.encode_to(dest);
			},
		}
	}
}

impl Decode for Era {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let first = input.read_byte()?;
		if first == 0 {
			Ok(Self::Immortal)
		} else {
			let encoded = first as u64 + ((input.read_byte()? as u64) << 8);
			let period = 2 << (encoded % (1 << 4));
			let quantize_factor = (period >> 12).max(1);
			let phase = (encoded >> 4) * quantize_factor;
			if period >= 4 && phase < period {
				Ok(Self::Mortal { period, phase })
			} else {
				Err("Invalid period and phase".into())
			}
		}
	}
}

#[derive(Clone)]
pub struct TransactionPayload<'a> {
	pub call: Cow<'a, TransactionCall>,
	pub extra: TransactionExtra,
	pub additional: TransactionAdditional,
}

impl<'a> TransactionPayload<'a> {
	pub fn new(call: TransactionCall, extra: TransactionExtra, additional: TransactionAdditional) -> Self {
		Self {
			call: Cow::Owned(call),
			extra,
			additional,
		}
	}

	pub fn new_borrowed(call: &'a TransactionCall, extra: TransactionExtra, additional: TransactionAdditional) -> Self {
		Self {
			call: Cow::Borrowed(call),
			extra,
			additional,
		}
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

#[derive(Clone)]
pub struct SignedTransaction {
	pub address: MultiAddress,
	pub signature: MultiSignature,
	pub tx_extra: TransactionExtra,
}

#[derive(Clone)]
pub struct Transaction<'a> {
	pub signed: Option<SignedTransaction>,
	pub payload: TransactionPayload<'a>,
}

impl<'a> Transaction<'a> {
	pub fn new(account_id: AccountId, signature: [u8; 64], payload: TransactionPayload<'a>) -> Self {
		let address = MultiAddress::Id(account_id);
		let signature = MultiSignature::Sr25519(signature);

		let signed = Some(SignedTransaction {
			address,
			signature,
			tx_extra: payload.extra.clone(),
		});

		Self { signed, payload }
	}

	pub fn encode(&self) -> Vec<u8> {
		let mut encoded_tx_inner = Vec::new();
		if let Some(signed) = &self.signed {
			0x84u8.encode_to(&mut encoded_tx_inner);
			signed.address.encode_to(&mut encoded_tx_inner);
			signed.signature.encode_to(&mut encoded_tx_inner);
			self.payload.extra.encode_to(&mut encoded_tx_inner);
		} else {
			0x4u8.encode_to(&mut encoded_tx_inner);
		}

		self.payload.call.encode_to(&mut encoded_tx_inner);
		let mut encoded_tx = Compact(encoded_tx_inner.len() as u32).encode();
		encoded_tx.append(&mut encoded_tx_inner);

		encoded_tx
	}
}

pub type AccountId = AccountId32;
pub type AccountIndex = u32;
pub type BlockNumber = u32;
pub type BlockHash = H256;

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
