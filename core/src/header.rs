use codec::{Compact, Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use subxt_core::config::{Hasher, Header, substrate::BlakeTwo256};

pub use subxt_core::config::substrate::{Digest, DigestItem};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct AvailHeader {
	pub parent_hash: H256,
	#[serde(serialize_with = "number_to_hex", deserialize_with = "number_from_hex")]
	#[codec(compact)]
	pub number: u32,
	pub state_root: H256,
	pub extrinsics_root: H256,
	pub digest: Digest,
	pub extension: HeaderExtension,
}

impl AvailHeader {
	pub fn data_root(&self) -> H256 {
		match &self.extension {
			HeaderExtension::V3(ext) => ext.commitment.data_root,
		}
	}

	pub fn hash(&self) -> H256 {
		BlakeTwo256::hash_of(self)
	}
}

impl Header for AvailHeader {
	type Hasher = BlakeTwo256;
	type Number = u32;

	fn number(&self) -> Self::Number {
		self.number
	}

	fn hash(&self) -> <Self::Hasher as Hasher>::Output {
		self.hash()
	}
}

fn number_to_hex<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let hex_string = format!("{:X}", value);
	serializer.serialize_str(&hex_string)
}

fn number_from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
	D: Deserializer<'de>,
{
	let buf = String::deserialize(deserializer)?;
	let without_prefix = buf.trim_start_matches("0x");
	let result = u32::from_str_radix(without_prefix, 16);
	match result {
		Ok(res) => Ok(res),
		Err(err) => Err(serde::de::Error::custom(err)),
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[repr(u8)]
pub enum HeaderExtension {
	V3(V3HeaderExtension) = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct V3HeaderExtension {
	pub app_lookup: CompactDataLookup,
	pub commitment: KateCommitment,
}
impl Encode for V3HeaderExtension {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		self.app_lookup.encode_to(dest);
		self.commitment.encode_to(dest);
	}
}
impl Decode for V3HeaderExtension {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let app_lookup = Decode::decode(input)?;
		let commitment = Decode::decode(input)?;
		Ok(Self { app_lookup, commitment })
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactDataLookup {
	// Compact
	pub size: u32,
	pub index: Vec<DataLookupItem>,
}
impl Encode for CompactDataLookup {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		Compact(self.size).encode_to(dest);
		self.index.encode_to(dest);
	}
}
impl Decode for CompactDataLookup {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let size = Compact::<u32>::decode(input)?.0;
		let index = Decode::decode(input)?;
		Ok(Self { size, index })
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataLookupItem {
	// Compact
	pub app_id: u32,
	// Compact
	pub start: u32,
}
impl Encode for DataLookupItem {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		Compact(self.app_id).encode_to(dest);
		Compact(self.start).encode_to(dest);
	}
}
impl Decode for DataLookupItem {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let app_id = Compact::<u32>::decode(input)?.0;
		let start = Compact::<u32>::decode(input)?.0;
		Ok(Self { app_id, start })
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KateCommitment {
	// Compact
	pub rows: u16,
	// Compact
	pub cols: u16,
	pub commitment: Vec<u8>,
	pub data_root: H256,
}
impl Encode for KateCommitment {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		Compact(self.rows).encode_to(dest);
		Compact(self.cols).encode_to(dest);
		self.commitment.encode_to(dest);
		self.data_root.encode_to(dest);
	}
}
impl Decode for KateCommitment {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let rows = Compact::<u16>::decode(input)?.0;
		let cols = Compact::<u16>::decode(input)?.0;
		let commitment = Decode::decode(input)?;
		let data_root = Decode::decode(input)?;
		Ok(Self { rows, cols, commitment, data_root })
	}
}
