use crate::chain_types::RuntimeEvent;
use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub trait HasEventEmittedIndex {
	// Pallet ID, Variant ID
	const EMITTED_INDEX: (u8, u8);
}

pub trait TransactionEventLike {
	fn from_raw(raw: &[u8]) -> Option<Box<Self>>;
}

impl<T: HasEventEmittedIndex + Encode + Decode> TransactionEventLike for T {
	fn from_raw(raw: &[u8]) -> Option<Box<T>> {
		if raw.len() < 2 {
			return None;
		}

		let (pallet_id, variant_id) = (raw[0], raw[1]);
		if Self::EMITTED_INDEX.0 != pallet_id || Self::EMITTED_INDEX.1 != variant_id {
			return None;
		}

		Self::decode(&mut &raw[2..]).ok().map(Box::new)
	}
}

/// Contains only the event body. Phase and topics are not included here.
#[derive(Debug, Clone)]
pub struct OpaqueEvent(pub Vec<u8>);

impl OpaqueEvent {
	pub fn pallet_index(&self) -> u8 {
		self.0[0]
	}

	pub fn variant_index(&self) -> u8 {
		self.0[1]
	}

	pub fn event_data(&self) -> &[u8] {
		&self.0[2..]
	}
}

impl TryFrom<String> for OpaqueEvent {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl TryFrom<&String> for OpaqueEvent {
	type Error = String;

	fn try_from(value: &String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl TryFrom<&str> for OpaqueEvent {
	type Error = String;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let value = hex::decode(value).map_err(|x| x.to_string())?;
		Self::try_from(value)
	}
}

impl TryFrom<Vec<u8>> for OpaqueEvent {
	type Error = String;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&Vec<u8>> for OpaqueEvent {
	type Error = String;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&[u8]> for OpaqueEvent {
	type Error = String;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		if value.len() < 3 {
			return Err("Event must have more than two bytes".into());
		}

		Ok(OpaqueEvent(value.to_owned()))
	}
}

impl TryFrom<OpaqueEvent> for RuntimeEvent {
	type Error = codec::Error;

	fn try_from(value: OpaqueEvent) -> Result<Self, Self::Error> {
		Self::try_from(&value)
	}
}

impl TryFrom<&OpaqueEvent> for RuntimeEvent {
	type Error = codec::Error;

	fn try_from(value: &OpaqueEvent) -> Result<Self, Self::Error> {
		RuntimeEvent::try_from(value.0.as_slice())
	}
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
