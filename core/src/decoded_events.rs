use crate::{HasHeader, types::metadata::StringOrBytes};
use codec::{Decode, Encode};

pub trait TransactionEventEncodable {
	/// SCALE encodes the event
	///
	/// If you need to Hex and SCALE encode then call `encode_as_hex_event`
	fn to_event(&self) -> Vec<u8>;
}

pub trait TransactionEventDecodable: Sized {
	/// Decodes the SCALE encoded Event
	///
	/// If you need to decode Hex string call `decode_hex_event`
	fn from_event<'a>(event: impl Into<StringOrBytes<'a>>) -> Result<Self, String>;
}

impl<T: HasHeader + Encode> TransactionEventEncodable for T {
	fn to_event(&self) -> Vec<u8> {
		let pallet_id = Self::HEADER_INDEX.0;
		let variant_id = Self::HEADER_INDEX.1;
		let mut encoded_event: Vec<u8> = vec![pallet_id, variant_id];
		Self::encode_to(self, &mut encoded_event);

		encoded_event
	}
}

impl<T: HasHeader + Decode> TransactionEventDecodable for T {
	fn from_event<'a>(event: impl Into<StringOrBytes<'a>>) -> Result<Self, String> {
		fn inner<T: HasHeader + Decode>(event: StringOrBytes) -> Result<T, String> {
			let event = match event {
				StringOrBytes::StringRef(s) => {
					&const_hex::decode(s.trim_start_matches("0x")).map_err(|x| x.to_string())?
				},
				StringOrBytes::String(s) => {
					&const_hex::decode(s.trim_start_matches("0x")).map_err(|x| x.to_string())?
				},
				StringOrBytes::Bytes(b) => b,
			};

			// This was moved out in order to decrease compilation times
			if !event_filter_in(event, T::HEADER_INDEX) {
				return Err("Failed to decode event. TODO".into());
			}

			if event.len() <= 2 {
				let mut data: &[u8] = &[];
				Ok(T::decode(&mut data).map_err(|x| x.to_string())?)
			} else {
				let mut data = &event[2..];
				Ok(T::decode(&mut data).map_err(|x| x.to_string())?)
			}
		}

		inner(event.into())
	}
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

/* /// Contains only the event body. Phase and topics are not included here.
#[derive(Debug, Clone)]
pub struct RawEvent(pub Vec<u8>);

impl RawEvent {
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

impl TryFrom<String> for RawEvent {
	type Error = String;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl TryFrom<&String> for RawEvent {
	type Error = String;

	fn try_from(value: &String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl TryFrom<&str> for RawEvent {
	type Error = String;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let value = const_hex::decode(value).map_err(|x| x.to_string())?;
		Self::try_from(value)
	}
}

impl TryFrom<Vec<u8>> for RawEvent {
	type Error = String;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&Vec<u8>> for RawEvent {
	type Error = String;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<&[u8]> for RawEvent {
	type Error = String;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		if value.len() < 2 {
			return Err("Event must have more than one byte".into());
		}

		Ok(RawEvent(value.to_owned()))
	}
}
 */
