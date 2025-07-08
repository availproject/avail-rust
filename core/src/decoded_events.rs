use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub trait HasEventEmittedIndex {
	// Pallet ID, Variant ID
	const EMITTED_INDEX: (u8, u8);
}

pub trait TransactionEventLike {
	fn decode_event(event: &[u8]) -> Option<Box<Self>>;
	fn decode_event_data(event_data: &[u8]) -> Option<Box<Self>>;
}

impl<T: HasEventEmittedIndex + Decode> TransactionEventLike for T {
	fn decode_event(event: &[u8]) -> Option<Box<T>> {
		// This was moved out in order to decrease compilation times
		if !event_filter_in(event, Self::EMITTED_INDEX) {
			return None;
		}

		if event.len() <= 2 {
			try_decode_event_data(&[])
		} else {
			try_decode_event_data(&event[2..])
		}
	}

	fn decode_event_data(event_data: &[u8]) -> Option<Box<T>> {
		// This was moved out in order to decrease compilation times
		try_decode_event_data(event_data)
	}
}

// Purely here to decrease compilation times
#[inline(never)]
fn try_decode_event_data<T: Decode>(mut event_data: &[u8]) -> Option<Box<T>> {
	T::decode(&mut event_data).ok().map(Box::new)
}

// Purely here to decrease compilation times
fn event_filter_in(event: &[u8], emitted_index: (u8, u8)) -> bool {
	if event.len() < 2 {
		return false;
	}

	let (pallet_id, variant_id) = (event[0], event[1]);
	if emitted_index.0 != pallet_id || emitted_index.1 != variant_id {
		return false;
	}

	true
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
