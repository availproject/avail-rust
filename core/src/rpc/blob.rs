use crate::RpcError;
use crate::rpc::kate::DataProof;
use codec::{Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use subxt_rpcs::{RpcClient, rpc_params};

/// A data blob
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Blob {
	/// The hash of the blob.
	pub blob_hash: H256,
	/// The actual data of this blob.
	pub data: Vec<u8>,
	/// The size of the blob.
	pub size: u64,
}

/// Ownership entry as returned by the node indexer / runtime summary.
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct OwnershipEntry {
	/// The address that owns the blob
	pub address: String,
	/// The babe key of the validator
	pub babe_key: String,
	/// The corresponding peer id
	pub encoded_peer_id: String,
	/// The signature of the holder
	pub signature: Vec<u8>,
}

/// BlobInfo returned by `blob_getBlobInfo`
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct BlobInfo {
	/// The hash of the blob.
	pub hash: H256,
	/// Block hash where this blob was observed (canonical/finalized).
	pub block_hash: H256,
	/// Block number where this blob was observed.
	pub block_number: u32,
	/// Ownership entries for the blob.
	pub ownership: Vec<OwnershipEntry>,
}

pub async fn submit_blob(client: &RpcClient, metadata_signed_transaction: &[u8], blob: &[u8]) -> Result<(), RpcError> {
	use base64::Engine;
	let encoded_metadata = base64::engine::general_purpose::STANDARD.encode(&metadata_signed_transaction);
	let encoded_blob = base64::engine::general_purpose::STANDARD.encode(&blob);

	let params = rpc_params![encoded_metadata, encoded_blob];
	let _value: () = client.request("blob_submitBlob", params).await?;
	Ok(())
}

pub async fn get_blob(
	client: &RpcClient,
	block_hash: H256,
	blob_index: u32,
	blob_hash: H256,
) -> Result<Blob, RpcError> {
	let params = rpc_params![block_hash, blob_index, blob_hash];
	let value: Blob = client.request("blob_getBlob", params).await?;
	Ok(value)
}

pub async fn get_blob_v2(client: &RpcClient, blob_hash: H256, block_hash: Option<H256>) -> Result<Blob, RpcError> {
	let params = rpc_params![blob_hash, block_hash];
	let value: Blob = client.request("blob_getBlobV2", params).await?;
	Ok(value)
}

/// Get canonical blob indexing info (if any) for a given blob_hash.
pub async fn get_blob_info(client: &RpcClient, blob_hash: H256) -> Result<BlobInfo, RpcError> {
	let params = rpc_params![blob_hash];
	let value: BlobInfo = client.request("blob_getBlobInfo", params).await?;
	Ok(value)
}

/// Get inclusion proof for a blob. If `at` is `Some(block_hash)` the proof
/// is generated for that block; if `None` the node will use its indexed finalised block.
pub async fn inclusion_proof(client: &RpcClient, blob_hash: H256, at: Option<H256>) -> Result<DataProof, RpcError> {
	let params = rpc_params![blob_hash, at];
	let value: DataProof = client.request("blob_inclusionProof", params).await?;
	Ok(value)
}

pub async fn get_blob(
	client: &RpcClient,
	block_hash: H256,
	blob_index: u32,
	blob_hash: H256,
) -> Result<Blob, RpcError> {
	let params = rpc_params![block_hash, blob_index, blob_hash];
	let value: Blob = client.request("blob_getBlob", params).await?;
	Ok(value)
}

pub async fn get_blob_v2(client: &RpcClient, blob_hash: H256, block_hash: Option<H256>) -> Result<Blob, RpcError> {
	let params = rpc_params![blob_hash, block_hash];
	let value: Blob = client.request("blob_getBlobV2", params).await?;
	Ok(value)
}

/// Get canonical blob indexing info (if any) for a given blob_hash.
pub async fn get_blob_info(client: &RpcClient, blob_hash: H256) -> Result<BlobInfo, RpcError> {
	let params = rpc_params![blob_hash];
	let value: BlobInfo = client.request("blob_getBlobInfo", params).await?;
	Ok(value)
}

/// Get inclusion proof for a blob. If `at` is `Some(block_hash)` the proof
/// is generated for that block; if `None` the node will use its indexed finalised block.
pub async fn inclusion_proof(client: &RpcClient, blob_hash: H256, at: Option<H256>) -> Result<DataProof, RpcError> {
	let params = rpc_params![blob_hash, at];
	let value: DataProof = client.request("blob_inclusionProof", params).await?;
	Ok(value)
}
