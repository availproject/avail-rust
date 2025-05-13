use crate::primitives::kate::{BlockLength, Cell, GDataProof, GRow, ProofResponse};
use primitive_types::H256;
use subxt_rpcs::rpc_params;
use subxt_rpcs::RpcClient;

pub async fn kate_block_length(client: &RpcClient, at: Option<H256>) -> Result<BlockLength, subxt_rpcs::Error> {
	let params = rpc_params![at];
	let value = client.request("kate_blockLength", params).await?;
	Ok(value)
}

pub async fn kate_query_data_proof(
	client: &RpcClient,
	transaction_index: u32,
	at: Option<H256>,
) -> Result<ProofResponse, subxt_rpcs::Error> {
	let params = rpc_params![transaction_index, at];
	let value = client.request("kate_queryDataProof", params).await?;
	Ok(value)
}

pub async fn kate_query_proof(
	client: &RpcClient,
	cells: Vec<Cell>,
	at: Option<H256>,
) -> Result<Vec<GDataProof>, subxt_rpcs::Error> {
	let params = rpc_params![cells, at];
	let value = client.request("kate_queryProof", params).await?;
	Ok(value)
}

pub async fn kate_query_rows(
	client: &RpcClient,
	rows: Vec<u32>,
	at: Option<H256>,
) -> Result<Vec<GRow>, subxt_rpcs::Error> {
	let params = rpc_params![rows, at];
	let value = client.request("kate_queryRows", params).await?;
	Ok(value)
}
