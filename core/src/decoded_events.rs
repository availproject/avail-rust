use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::HasHeader;

pub trait TransactionEventEncodable {
	/// SCALE encodes the event
	///
	/// If you need to Hex and SCALE encode then call `encode_as_hex_event`
	fn encode_as_event(&self) -> Vec<u8>;

	/// Hex and SCALE encodes the event
	///
	/// If you need to just SCALE encode then call `encode_as_event`
	fn encode_as_hex_event(&self) -> String;
}

pub trait TransactionEventDecodable: Sized {
	/// Decodes the SCALE encoded Event
	///
	/// If you need to decode Hex string call `decode_hex_event`
	fn decode_event(event: &[u8]) -> Option<Self>;

	/// Decodes the Hex and SCALE encoded Transaction Call
	/// This is equal to Hex::decode + Self::decode_event
	///
	/// If you need to decode bytes call `decode_event`
	fn decode_hex_event(event: &str) -> Option<Self>;

	/// Decodes the SCALE encoded Event Data
	fn decode_event_data(event_data: &[u8]) -> Option<Self>;
}

impl<T: HasHeader + Encode> TransactionEventEncodable for T {
	fn encode_as_event(&self) -> Vec<u8> {
		let pallet_id = Self::HEADER_INDEX.0;
		let variant_id = Self::HEADER_INDEX.1;
		let mut encoded_event: Vec<u8> = vec![pallet_id, variant_id];
		Self::encode_to(self, &mut encoded_event);

		encoded_event
	}

	fn encode_as_hex_event(&self) -> String {
		std::format!("0x{}", const_hex::encode(Self::encode_as_event(self)))
	}
}

impl<T: HasHeader + Decode> TransactionEventDecodable for T {
	fn decode_event(event: &[u8]) -> Option<T> {
		// This was moved out in order to decrease compilation times
		if !event_filter_in(event, Self::HEADER_INDEX) {
			return None;
		}

		if event.len() <= 2 {
			try_decode_event_data(&[])
		} else {
			try_decode_event_data(&event[2..])
		}
	}

	#[inline(always)]
	fn decode_hex_event(event: &str) -> Option<T> {
		let hex_decoded = const_hex::decode(event.trim_start_matches("0x")).ok()?;
		Self::decode_event(&hex_decoded)
	}

	fn decode_event_data(event_data: &[u8]) -> Option<T> {
		// This was moved out in order to decrease compilation times
		try_decode_event_data(event_data)
	}
}

// Purely here to decrease compilation times
#[inline(never)]
fn try_decode_event_data<T: Decode>(mut event_data: &[u8]) -> Option<T> {
	T::decode(&mut event_data).ok()
}

// Purely here to decrease compilation times
#[inline(never)]
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
		if self.0.len() <= 2 { &[] } else { &self.0[2..] }
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
		let value = const_hex::decode(value).map_err(|x| x.to_string())?;
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
		if value.len() < 2 {
			return Err("Event must have more than one byte".into());
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
