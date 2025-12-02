use codec::{Compact, Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use subxt_core::config::{substrate::BlakeTwo256, Hasher, Header as SubxtHeader};

pub use subxt_core::config::substrate::{Digest, DigestItem};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Encode, Decode)]
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
	/// Data root of all DA data in this block, regardless of PCS (KZG/Fri).
	pub fn data_root(&self) -> H256 {
		match &self.extension {
			HeaderExtension::Kzg(KzgHeader::V4(ext)) => ext.commitment.data_root,
			HeaderExtension::Fri(FriHeader::V1(ext)) => ext.data_root,
		}
	}

	pub fn hash(&self) -> H256 {
		BlakeTwo256::hash_of(self)
	}
}

impl SubxtHeader for AvailHeader {
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

/// Top-level DA header extension: *which PCS + which version inside*.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
// #[serde(rename_all = "camelCase")]
pub enum HeaderExtension {
	/// KZG-based DA header (current mainnet scheme, v4).
	Kzg(KzgHeader),
	/// Fri/Binius-based DA header (new scheme).
	Fri(FriHeader),
}

impl Default for HeaderExtension {
	fn default() -> Self {
		HeaderExtension::Fri(FriHeader::V1(FriV1HeaderExtension::default()))
	}
}

/// KZG header variants (only v4 is used on-chain now).
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum KzgHeader {
	V4(V4HeaderExtension),
}

/// Fri header variants (v1 for now).
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum FriHeader {
	V1(FriV1HeaderExtension),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct V4HeaderExtension {
	pub app_lookup: V4CompactDataLookup,
	pub commitment: KateCommitment,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct V4CompactDataLookup {
	#[codec(compact)]
	pub size: u32,
	pub index: Vec<DataLookupItem>,
	pub rows_per_tx: Vec<u16>,
}

/// Fri blob commitment: one entry per blob in the block.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, Default)]
#[serde(rename_all = "camelCase")]
pub struct FriBlobCommitment {
	/// Blob size in bytes (original data).
	pub size_bytes: u64,
	/// Commitment to the encoded blob (Merkle root, 32 bytes).
	pub commitment: H256,
}

/// Version tag for Fri parameters.
/// This mirrors `FriParamsVersion(pub u8)` on-chain.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, Default)]
#[serde(rename_all = "camelCase")]
pub struct FriParamsVersion(pub u8);

/// Fri v1 header extension: aggregate of all blob commitments for the block.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, Default)]
#[serde(rename_all = "camelCase")]
pub struct FriV1HeaderExtension {
	pub blobs: Vec<FriBlobCommitment>,
	pub data_root: H256,
	pub params_version: FriParamsVersion,
}
