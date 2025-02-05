use crate::{
	error::ClientError,
	from_substrate::{FeeDetails, RuntimeDispatchInfo},
	rpc, Client, H256,
};
use codec::Decode;

pub mod transaction_payment {
	use super::*;

	pub async fn query_info(
		client: &Client,
		mut extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, ClientError> {
		let len = extrinsic.len() as u32;
		let bytes = len.to_ne_bytes();
		extrinsic.extend_from_slice(&bytes);

		let result = rpc::state::call(client, "TransactionPaymentApi_query_info", &extrinsic, at).await?;
		let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		let result = RuntimeDispatchInfo::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

		Ok(result)
	}

	pub async fn query_fee_details(
		client: &Client,
		mut extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, ClientError> {
		let len = extrinsic.len() as u32;
		let bytes = len.to_ne_bytes();
		extrinsic.extend_from_slice(&bytes);

		let result = rpc::state::call(client, "TransactionPaymentApi_query_fee_details", &extrinsic, at).await?;
		let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		let result = FeeDetails::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

		Ok(result)
	}

	pub async fn query_call_info(
		client: &Client,
		mut call: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, ClientError> {
		let len = call.len() as u32;
		let bytes = len.to_ne_bytes();
		call.extend_from_slice(&bytes);

		let result = rpc::state::call(client, "TransactionPaymentCallApi_query_call_info", &call, at).await?;
		let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		let result = RuntimeDispatchInfo::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

		Ok(result)
	}

	pub async fn query_call_fee_details(
		client: &Client,
		mut call: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, ClientError> {
		let len = call.len() as u32;
		let bytes = len.to_ne_bytes();
		call.extend_from_slice(&bytes);

		let result = rpc::state::call(client, "TransactionPaymentCallApi_query_call_fee_details", &call, at).await?;
		let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		let result = FeeDetails::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

		Ok(result)
	}
}
