use subxt_rpcs::{rpc_params, RpcClient};

pub async fn submit_blob(
	client: &RpcClient,
	metadata_signed_transaction: Vec<u8>,
	blob: Vec<u8>,
) -> Result<(), subxt_rpcs::Error> {
	let params = rpc_params![metadata_signed_transaction, blob];
	let _value: () = client.request("blob_submitBlob", params).await?;
	Ok(())
}
