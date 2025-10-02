use crate::RpcError;
use subxt_rpcs::{RpcClient, rpc_params};

pub async fn submit_blob(client: &RpcClient, metadata_signed_transaction: &[u8], blob: &[u8]) -> Result<(), RpcError> {
	use base64::Engine;
	let encoded_metadata = base64::engine::general_purpose::STANDARD.encode(&metadata_signed_transaction);
	let encoded_blob = base64::engine::general_purpose::STANDARD.encode(&blob);

	let params = rpc_params![encoded_metadata, encoded_blob];
	let _value: () = client.request("blob_submitBlob", params).await?;
	Ok(())
}
