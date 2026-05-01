use codec::{Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use subxt_core::config::{Hasher, Header as SubxtHeader, substrate::BlakeTwo256};

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
	/// Data root of all DA blobs & bridge txs in this block.
	pub fn data_root(&self) -> H256 {
		match &self.extension {
			HeaderExtension::V1(ext) => ext.data_root,
		}
	}

	pub fn hash(&self) -> H256 {
		BlakeTwo256.hash_of(self)
	}
}

impl SubxtHeader for AvailHeader {
	type Hasher = BlakeTwo256;
	type Number = u32;

	fn number(&self) -> Self::Number {
		self.number
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

/// Versioned DA header extension.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum HeaderExtension {
	/// FRI/Binius-based DA header.
	V1(FriV1HeaderExtension),
}

impl Default for HeaderExtension {
	fn default() -> Self {
		HeaderExtension::V1(FriV1HeaderExtension::default())
	}
}

/// Fri blob commitment: one entry per blob in the block.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, Default)]
#[serde(rename_all = "camelCase")]
pub struct FriBlobCommitment {
	/// Blob hash.
	pub blob_hash: H256,
	/// Blob size in bytes (original data).
	pub size_bytes: u64,
	/// FRI PCS commitment.
	pub commitment: Vec<u8>,
}

/// Version tag for Fri parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, Default)]
pub enum FriParamsVersion {
	#[default]
	V0,
}

/// Fri v1 header extension: aggregate of all blob commitments for the block.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, Default)]
#[serde(rename_all = "camelCase")]
pub struct FriV1HeaderExtension {
	pub blobs: Vec<FriBlobCommitment>,
	pub data_root: H256,
	pub params_version: FriParamsVersion,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn header_extension_v1_uses_variant_index_zero() {
		let extension = HeaderExtension::V1(FriV1HeaderExtension::default());

		assert_eq!(extension.encode()[0], 0);
	}

	#[test]
	fn fri_blob_commitment_matches_core_field_order() {
		let blob_hash = H256::repeat_byte(1);
		let commitment = vec![2, 3, 5, 8];
		let blob = FriBlobCommitment { blob_hash, size_bytes: 13, commitment: commitment.clone() };
		let decoded = FriBlobCommitment::decode(&mut &blob.encode()[..]).expect("valid encoding");

		assert_eq!(decoded.blob_hash, blob_hash);
		assert_eq!(decoded.size_bytes, 13);
		assert_eq!(decoded.commitment, commitment);
	}
}
