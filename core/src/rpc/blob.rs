use crate::RpcError;
use subxt_rpcs::{RpcClient, rpc_params};

pub async fn submit_blob(
	client: &RpcClient,
	metadata_signed_transaction: Vec<u8>,
	blob: Vec<u8>,
) -> Result<(), RpcError> {
	let hex_encoded_metadata_signed_transaction = const_hex::encode(metadata_signed_transaction);
	let hex_encoded_blob = const_hex::encode(blob);
	let params = rpc_params![hex_encoded_metadata_signed_transaction, hex_encoded_blob];
	let _value: () = client.request("blob_submitBlob", params).await?;
	Ok(())
}
