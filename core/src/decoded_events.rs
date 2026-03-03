use crate::{HasHeader, types::metadata::StringOrBytes};
use codec::{Decode, Encode};
use scale_value::{Composite, Value, ValueDef, scale::decode_as_type};
use subxt_core::events::Phase;
use subxt_metadata::{Metadata, StorageEntryType};

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
			let event: &[u8] = match &event {
				StringOrBytes::StringRef(s) => {
					&const_hex::decode(s.trim_start_matches("0x")).map_err(|x| x.to_string())?
				},
				StringOrBytes::BoxedString(s) => {
					&const_hex::decode(s.trim_start_matches("0x")).map_err(|x| x.to_string())?
				},
				StringOrBytes::Bytes(b) => b,
				StringOrBytes::BoxedBytes(b) => b,
			};

			// This was moved out in order to decrease compilation times
			check_header(event, T::HEADER_INDEX)?;

			let mut data = if event.len() <= 2 { &[] } else { &event[2..] };
			T::decode(&mut data).map_err(|x| x.to_string())
		}

		inner(event.into())
	}
}

// Purely here to decrease compilation times
pub(crate) fn check_header(data: &[u8], header_index: (u8, u8)) -> Result<(), String> {
	if data.len() < 2 {
		return Err("Failed to decode. Not have enough bytes to decode the header".into());
	}

	let (pallet_id, variant_id) = (data[0], data[1]);
	if header_index.0 != pallet_id || header_index.1 != variant_id {
		let err = std::format!(
			"Failed to decode. Mismatch in pallet and/or variant id. Actual: PI: {}, VI: {} Expected: PI: {}, VI: {}",
			pallet_id,
			variant_id,
			header_index.0,
			header_index.1
		);
		return Err(err);
	}

	Ok(())
}

pub fn parse_encoded_events(metadata: &Metadata, mut raw_bytes: &[u8]) -> Option<Vec<EncodedEvent>> {
	let system = metadata.pallet_by_name("System").unwrap();
	let events = system.storage().unwrap().entry_by_name("Events").unwrap();

	let type_id = match events.entry_type() {
		StorageEntryType::Plain(ty) => *ty,
		_ => return None,
	};

	let registry = metadata.types();
	let value = decode_as_type(&mut raw_bytes, type_id, registry).unwrap();

	let ValueDef::Composite(x) = &value.value else {
		return None;
	};

	let mut encoded_events = Vec::new();
	let values = x.values();
	for val in values {
		let ValueDef::Composite(x) = &val.value else {
			return None;
		};

		if let Composite::Named(x) = x {
			let mut event = EncodedEvent::default();
			for som in x {
				if som.0 == "phase" {
					event.encoded_phase = Some(som.1.clone());
				}

				if som.0 == "event" {
					event.encoded_event = Some(som.1.clone());
				}

				if som.0 == "topics" {
					event.encoded_topics = Some(som.1.clone());
				}
			}
			encoded_events.push(event);
		}
	}

	Some(encoded_events)
}

#[derive(Debug, Clone, Default)]
pub struct EncodedEvent {
	pub encoded_phase: Option<Value<u32>>,
	pub encoded_event: Option<Value<u32>>,
	pub encoded_topics: Option<Value<u32>>,
}

impl EncodedEvent {
	pub fn decode_phase(&self) -> Option<Phase> {
		let value = self.encoded_phase.as_ref()?;

		decode_phase(value)
	}

	pub fn decode_pallet_variant_name(&self) -> Option<(String, String)> {
		let value = self.encoded_event.as_ref()?;

		decode_pallet_variant_name(value)
	}
}

pub fn decode_phase(value: &Value<u32>) -> Option<Phase> {
	let ValueDef::Variant(value) = &value.value else {
		return None;
	};

	if value.name == "Initialization" {
		return Some(Phase::Initialization);
	}
	if value.name == "Finalization" {
		return Some(Phase::Finalization);
	}
	if value.name != "ApplyExtrinsic" {
		return None;
	}

	let values = match &value.values {
		Composite::Named(_) => return None,
		Composite::Unnamed(values) => values,
	};

	let value = values.first()?;
	let ext_index = value.as_u128()?;

	Some(Phase::ApplyExtrinsic(ext_index as u32))
}

/// Return pallet and variant name
pub fn decode_pallet_variant_name(value: &Value<u32>) -> Option<(String, String)> {
	let ValueDef::Variant(value) = &value.value else {
		return None;
	};

	let pallet_name = value.name.clone();

	let Composite::Unnamed(values) = &value.values else {
		return None;
	};

	let value = values.first()?;

	let ValueDef::Variant(value) = &value.value else {
		return None;
	};

	let variant_name = value.name.clone();

	Some((pallet_name, variant_name))
}
