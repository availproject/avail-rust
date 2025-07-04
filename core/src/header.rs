use codec::{Decode, Encode};
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
			HeaderExtension::V4(ext) => ext.commitment.data_root,
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

/* #[cfg(feature = "generated_metadata")]
pub mod with_subxt_metadata {
	use super::*;
	pub use crate::subxt_avail::runtime_types::{
		avail_core::header::{extension::HeaderExtension, Header as ApiHeader},
		sp_runtime::generic::digest::{Digest as ApiDigest, DigestItem as ApiDigestItem},
	};
	use avail_rust_core::marker::PhantomData;

	impl<B, H> From<AvailHeader> for ApiHeader<B, H>
	where
		B: From<u32>,
	{
		fn from(h: AvailHeader) -> Self {
			Self {
				parent_hash: h.parent_hash,
				number: h.number.into(),
				state_root: h.state_root,
				extrinsics_root: h.extrinsics_root,
				digest: h.digest.into(),
				extension: h.extension,
				__ignore: PhantomData,
			}
		}
	}

	impl From<Digest> for ApiDigest {
		fn from(d: Digest) -> Self {
			let logs = d.logs.into_iter().map(|xt_item| xt_item.into()).collect::<Vec<_>>();
			Self { logs }
		}
	}

	impl From<DigestItem> for ApiDigestItem {
		fn from(di: DigestItem) -> Self {
			match di {
				DigestItem::PreRuntime(id, data) => ApiDigestItem::PreRuntime(id, data),
				DigestItem::Consensus(id, data) => ApiDigestItem::Consensus(id, data),
				DigestItem::Seal(id, data) => ApiDigestItem::Seal(id, data),
				DigestItem::Other(data) => ApiDigestItem::Other(data),
				DigestItem::RuntimeEnvironmentUpdated => ApiDigestItem::RuntimeEnvironmentUpdated,
			}
		}
	}
}
	*/

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[repr(u8)]
pub enum HeaderExtension {
	V3(V3HeaderExtension) = 2,
	V4(V4HeaderExtension) = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct V3HeaderExtension {
	pub app_lookup: CompactDataLookup,
	pub commitment: KateCommitment,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct CompactDataLookup {
	#[codec(compact)]
	pub size: u32,
	pub index: Vec<DataLookupItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct DataLookupItem {
	#[codec(compact)]
	pub app_id: u32,
	#[codec(compact)]
	pub start: u32,
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

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct KateCommitment {
	#[codec(compact)]
	pub rows: u16,
	#[codec(compact)]
	pub cols: u16,
	pub commitment: Vec<u8>,
	pub data_root: H256,
}
