use subxt_rpcs::{RpcClient, rpc_params};

pub async fn block_justification(client: &RpcClient, at: u32) -> Result<Option<Vec<u8>>, subxt_rpcs::Error> {
	let params = rpc_params![at];
	let value = client.request("grandpa_blockJustification", params).await?;
	Ok(value)
}
