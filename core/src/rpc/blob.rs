use subxt_rpcs::{rpc_params, RpcClient};

pub async fn submit_blob(
	client: &RpcClient,
	metadata_signed_transaction: Vec<u8>,
	blob: Vec<u8>,
) -> Result<(), subxt_rpcs::Error> {
	let hex_encoded_metadata_signed_transaction = hex::encode(metadata_signed_transaction);
	let hex_encoded_blob = hex::encode(blob);
	let params = rpc_params![hex_encoded_metadata_signed_transaction, hex_encoded_blob];
	let _value: () = client.request("blob_submitBlob", params).await?;
	Ok(())
}
