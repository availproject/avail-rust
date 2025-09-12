use crate::{
	error::Error,
	types::substrate::{FeeDetails, RuntimeDispatchInfo},
};
use primitive_types::H256;
use subxt_rpcs::RpcClient;

pub async fn call_raw<T: codec::Decode>(
	client: &RpcClient,
	method: &str,
	data: &[u8],
	at: Option<H256>,
) -> Result<T, Error> {
	let result: String = super::state::call(client, method, data, at).await?;
	let result = const_hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
	let result = T::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

	Ok(result)
}

pub async fn api_transaction_payment_query_info(
	client: &RpcClient,
	mut extrinsic: Vec<u8>,
	at: Option<H256>,
) -> Result<RuntimeDispatchInfo, Error> {
	let len = extrinsic.len() as u32;
	let bytes = len.to_ne_bytes();
	extrinsic.extend_from_slice(&bytes);

	call_raw(client, "TransactionPaymentApi_query_info", &extrinsic, at).await
}

pub async fn api_transaction_payment_query_fee_details(
	client: &RpcClient,
	mut extrinsic: Vec<u8>,
	at: Option<H256>,
) -> Result<FeeDetails, Error> {
	let len = extrinsic.len() as u32;
	let bytes = len.to_ne_bytes();
	extrinsic.extend_from_slice(&bytes);

	call_raw(client, "TransactionPaymentApi_query_fee_details", &extrinsic, at).await
}

pub async fn api_transaction_payment_query_call_info(
	client: &RpcClient,
	mut call: Vec<u8>,
	at: Option<H256>,
) -> Result<RuntimeDispatchInfo, Error> {
	let len = call.len() as u32;
	let bytes = len.to_ne_bytes();
	call.extend_from_slice(&bytes);

	call_raw(client, "TransactionPaymentCallApi_query_call_info", &call, at).await
}

pub async fn api_transaction_payment_query_call_fee_details(
	client: &RpcClient,
	mut call: Vec<u8>,
	at: Option<H256>,
) -> Result<FeeDetails, Error> {
	let len = call.len() as u32;
	let bytes = len.to_ne_bytes();
	call.extend_from_slice(&bytes);

	call_raw(client, "TransactionPaymentCallApi_query_call_fee_details", &call, at).await
}
