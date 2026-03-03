use crate::{decoded_events::check_header, types::metadata::StringOrBytes};
use codec::{Decode, Encode};

pub trait HasHeader {
	// Pallet ID, Variant ID
	const HEADER_INDEX: (u8, u8);
}

pub trait TransactionEncodable {
	/// SCALE encodes the event
	///
	/// If you need to Hex and SCALE encode then call `encode_as_hex_event`
	fn to_call(&self) -> Vec<u8>;
}

pub trait ExtrinsicDecodable: Sized {
	fn from_call<'a>(call: impl Into<StringOrBytes<'a>>) -> Result<Self, String>;
	fn from_ext<'a>(call: impl Into<StringOrBytes<'a>>) -> Result<Self, String>;
}

impl<T: HasHeader + Encode> TransactionEncodable for T {
	fn to_call(&self) -> Vec<u8> {
		let pallet_id = Self::HEADER_INDEX.0;
		let variant_id = Self::HEADER_INDEX.1;
		let mut encoded_event: Vec<u8> = vec![pallet_id, variant_id];
		Self::encode_to(self, &mut encoded_event);

		encoded_event
	}
}

impl<T: HasHeader + Decode> ExtrinsicDecodable for T {
	fn from_call<'a>(call: impl Into<StringOrBytes<'a>>) -> Result<T, String> {
		fn inner<T: HasHeader + Decode>(call: StringOrBytes) -> Result<T, String> {
			let call: &[u8] = match &call {
				StringOrBytes::StringRef(s) => {
					&const_hex::decode(s.trim_start_matches("0x")).map_err(|x| x.to_string())?
				},
				StringOrBytes::BoxedString(s) => {
					&const_hex::decode(s.trim_start_matches("0x")).map_err(|x| x.to_string())?
				},
				StringOrBytes::Bytes(b) => b,
				StringOrBytes::BoxedBytes(b) => b,
			};

			check_header(call, T::HEADER_INDEX)?;

			let mut data = if call.len() <= 2 { &[] } else { &call[2..] };
			T::decode(&mut data).map_err(|x| x.to_string())
		}

		inner(call.into())
	}

	fn from_ext<'a>(ext: impl Into<StringOrBytes<'a>>) -> Result<T, String> {
		fn inner<T: HasHeader + Decode>(ext: StringOrBytes) -> Result<T, String> {
			let ext: &[u8] = match &ext {
				StringOrBytes::StringRef(s) => &const_hex::decode(s.trim_start_matches("0x"))
					.map_err(|x: const_hex::FromHexError| x.to_string())?,
				StringOrBytes::BoxedString(s) => {
					&const_hex::decode(s.trim_start_matches("0x")).map_err(|x| x.to_string())?
				},
				StringOrBytes::Bytes(b) => b,
				StringOrBytes::BoxedBytes(b) => b,
			};

			let ext = crate::substrate::Extrinsic::try_from(ext).map_err(|e| e.to_string())?;
			let mut inp = ext.call.0.as_slice();
			T::decode(&mut inp).map_err(|e| e.to_string())
		}

		inner(ext.into())
	}
}
