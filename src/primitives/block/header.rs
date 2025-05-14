use codec::{Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use subxt_core::config::{
	substrate::{BlakeTwo256, Digest},
	Hasher, Header,
};

#[cfg(feature = "subxt_metadata")]
pub use with_subxt_metadata::{HeaderExtension, *};

#[cfg(not(feature = "subxt_metadata"))]
pub use no_subxt::HeaderExtension;

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
}

impl Header for AvailHeader {
	type Hasher = BlakeTwo256;
	type Number = u32;

	fn number(&self) -> Self::Number {
		self.number
	}

	fn hash(&self) -> <Self::Hasher as Hasher>::Output {
		Self::Hasher::hash_of(self)
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
	// TODO
	Ok(u32::from_str_radix(without_prefix, 16).unwrap())
}

#[cfg(feature = "subxt_metadata")]
pub mod with_subxt_metadata {
	use super::*;
	pub use crate::subxt_avail::runtime_types::{
		avail_core::header::{extension::HeaderExtension, Header as ApiHeader},
		sp_runtime::generic::digest::{Digest as ApiDigest, DigestItem as ApiDigestItem},
	};
	use core::marker::PhantomData;
	pub use subxt_core::config::substrate::DigestItem;

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

pub mod no_subxt {
	pub use super::*;

	#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
	#[repr(u8)]
	pub enum HeaderExtension {
		V3(V3HeaderExtension) = 2,
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
	pub struct KateCommitment {
		#[codec(compact)]
		pub rows: u16,
		#[codec(compact)]
		pub cols: u16,
		pub commitment: Vec<u8>,
		pub data_root: H256,
	}

	#[derive(Debug, Clone)]
	#[repr(u8)]
	pub enum DigestItem {
		PreRuntime([u8; 4usize], Vec<u8>) = 6,
		Consensus([u8; 4usize], Vec<u8>) = 4,
		Seal([u8; 4usize], Vec<u8>) = 5,
		Other(Vec<u8>) = 0,
		RuntimeEnvironmentUpdated = 8,
	}
}
