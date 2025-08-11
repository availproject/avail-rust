use subxt_rpcs::{RpcClient, rpc_params};

pub async fn block_justification(client: &RpcClient, at: u32) -> Result<Option<String>, subxt_rpcs::Error> {
	let params = rpc_params![at];
	let value = client.request("grandpa_blockJustification", params).await?;
	Ok(value)
}

pub async fn block_justification_json(client: &RpcClient, at: u32) -> Result<Option<String>, subxt_rpcs::Error> {
	let params = rpc_params![at];
	let value: Box<serde_json::value::RawValue> = client.request("grandpa_blockJustificationJson", params).await?;
	let value = value.to_string();
	if value == "null" {
		return Ok(None);
	}

	Ok(Some(value))
}
