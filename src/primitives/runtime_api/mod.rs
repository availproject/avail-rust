use super::rpc::substrate;
use crate::{
	error::RpcError,
	from_substrate::{FeeDetails, RuntimeDispatchInfo},
};
use codec::Decode;
use primitive_types::H256;
use subxt_rpcs::RpcClient;

pub async fn api_transaction_payment_query_info(
	client: &RpcClient,
	mut extrinsic: Vec<u8>,
	at: Option<H256>,
) -> Result<RuntimeDispatchInfo, RpcError> {
	let len = extrinsic.len() as u32;
	let bytes = len.to_ne_bytes();
	extrinsic.extend_from_slice(&bytes);

	let result = substrate::state_call(client, "TransactionPaymentApi_query_info", &extrinsic, at).await?;
	let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
	let result = RuntimeDispatchInfo::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

	Ok(result)
}

pub async fn api_transaction_payment_query_fee_details(
	client: &RpcClient,
	mut extrinsic: Vec<u8>,
	at: Option<H256>,
) -> Result<FeeDetails, RpcError> {
	let len = extrinsic.len() as u32;
	let bytes = len.to_ne_bytes();
	extrinsic.extend_from_slice(&bytes);

	let result = substrate::state_call(client, "TransactionPaymentApi_query_fee_details", &extrinsic, at).await?;
	let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
	let result = FeeDetails::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

	Ok(result)
}

pub async fn api_transaction_payment_query_call_info(
	client: &RpcClient,
	mut call: Vec<u8>,
	at: Option<H256>,
) -> Result<RuntimeDispatchInfo, RpcError> {
	let len = call.len() as u32;
	let bytes = len.to_ne_bytes();
	call.extend_from_slice(&bytes);

	let result = substrate::state_call(client, "TransactionPaymentCallApi_query_call_info", &call, at).await?;
	let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
	let result = RuntimeDispatchInfo::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

	Ok(result)
}

pub async fn api_transaction_payment_query_call_fee_details(
	client: &RpcClient,
	mut call: Vec<u8>,
	at: Option<H256>,
) -> Result<FeeDetails, RpcError> {
	let len = call.len() as u32;
	let bytes = len.to_ne_bytes();
	call.extend_from_slice(&bytes);

	let result = substrate::state_call(client, "TransactionPaymentCallApi_query_call_fee_details", &call, at).await?;
	let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
	let result = FeeDetails::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

	Ok(result)
}
