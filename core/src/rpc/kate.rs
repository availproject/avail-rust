use crate::avail;
use codec::{Decode, Encode};
use primitive_types::{H256, U256};
use serde::{Deserialize, Serialize};
use subxt_rpcs::{RpcClient, rpc_params};

/// Compatible with `kate::com::Cell`
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Cell {
	#[codec(compact)]
	pub row: u32,
	#[codec(compact)]
	pub col: u32,
}

impl<R, C> From<(R, C)> for Cell
where
	R: Into<u32>,
	C: Into<u32>,
{
	fn from((row, col): (R, C)) -> Self {
		Self { row: row.into(), col: col.into() }
	}
}

pub type GRawScalar = U256;
pub type GRow = Vec<GRawScalar>;
pub type GDataProof = (GRawScalar, GProof);
pub type GMultiProof = (Vec<GRawScalar>, GProof);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "Vec<u8>", into = "Vec<u8>")]
pub struct GProof(pub [u8; 48]);

impl From<GProof> for Vec<u8> {
	fn from(proof: GProof) -> Self {
		proof.0.to_vec()
	}
}

impl TryFrom<Vec<u8>> for GProof {
	type Error = u32;

	fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
		if data.len() != 48 {
			return Err(data.len() as u32);
		};

		let mut proof = [0u8; 48];
		proof.copy_from_slice(&data);
		Ok(GProof(proof))
	}
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PerDispatchClassU32 {
	pub normal: u32,
	pub operational: u32,
	pub mandatory: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockLength {
	pub max: PerDispatchClassU32,
	pub cols: u32,
	pub rows: u32,
	pub chunk_size: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofResponse {
	pub data_proof: DataProof,
	pub message: Option<avail::vector::types::AddressedMessage>,
}

/// Wrapper of `binary-merkle-tree::MerkleProof` with codec support.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataProof {
	pub roots: TxDataRoots,
	/// Proof items (does not contain the leaf hash, nor the root obviously).
	///
	/// This vec contains all inner node hashes necessary to reconstruct the root hash given the
	/// leaf hash.
	pub proof: Vec<H256>,
	/// Number of leaves in the original tree.
	///
	/// This is needed to detect a case where we have an odd number of leaves that "get promoted"
	/// to upper layers.
	pub number_of_leaves: u32,
	/// Index of the leaf the proof is for (0-based).
	pub leaf_index: u32,
	/// Leaf content.
	pub leaf: H256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxDataRoots {
	/// Global Merkle root
	pub data_root: H256,
	/// Merkle root hash of submitted data
	pub blob_root: H256,
	/// Merkle root of bridged data
	pub bridge_root: H256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct GCellBlock {
	pub start_x: u32,
	pub start_y: u32,
	pub end_x: u32,
	pub end_y: u32,
}

pub async fn block_length(client: &RpcClient, at: Option<H256>) -> Result<BlockLength, subxt_rpcs::Error> {
	let params = rpc_params![at];
	let value = client.request("kate_blockLength", params).await?;
	Ok(value)
}

pub async fn query_data_proof(
	client: &RpcClient,
	transaction_index: u32,
	at: Option<H256>,
) -> Result<ProofResponse, subxt_rpcs::Error> {
	let params = rpc_params![transaction_index, at];
	let value = client.request("kate_queryDataProof", params).await?;
	Ok(value)
}

pub async fn query_proof(
	client: &RpcClient,
	cells: Vec<Cell>,
	at: Option<H256>,
) -> Result<Vec<GDataProof>, subxt_rpcs::Error> {
	let params = rpc_params![cells, at];
	let value = client.request("kate_queryProof", params).await?;
	Ok(value)
}

/// Constraint: You can pass up to 64 rows
pub async fn query_rows(client: &RpcClient, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, subxt_rpcs::Error> {
	let params = rpc_params![rows, at];
	let value = client.request("kate_queryRows", params).await?;
	Ok(value)
}

pub async fn query_multi_proof(
	client: &RpcClient,
	at: Option<H256>,
	cells: Vec<Cell>,
) -> Result<Vec<(GMultiProof, GCellBlock)>, subxt_rpcs::Error> {
	let params = rpc_params![cells.to_vec(), at];
	let proofs: Vec<(GMultiProof, GCellBlock)> = client.request("kate_queryMultiProof", params).await?;

	Ok(proofs)
}
