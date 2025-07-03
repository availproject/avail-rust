use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::chain_types::RuntimeEvent;

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
