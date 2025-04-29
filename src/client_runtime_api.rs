use crate::{
	error::ClientError,
	from_substrate::{FeeDetails, RuntimeDispatchInfo},
	Client, H256,
};
use codec::Decode;

impl Client {
	pub async fn api_transaction_payment_query_info(
		&self,
		mut extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, ClientError> {
		let len = extrinsic.len() as u32;
		let bytes = len.to_ne_bytes();
		extrinsic.extend_from_slice(&bytes);

		let result = self
			.rpc_state_call("TransactionPaymentApi_query_info", &extrinsic, at)
			.await?;
		let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		let result = RuntimeDispatchInfo::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

		Ok(result)
	}

	pub async fn api_transaction_payment_query_fee_details(
		&self,
		mut extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, ClientError> {
		let len = extrinsic.len() as u32;
		let bytes = len.to_ne_bytes();
		extrinsic.extend_from_slice(&bytes);

		let result = self
			.rpc_state_call("TransactionPaymentApi_query_fee_details", &extrinsic, at)
			.await?;
		let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		let result = FeeDetails::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

		Ok(result)
	}

	pub async fn api_transaction_payment_query_call_info(
		&self,
		mut call: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, ClientError> {
		let len = call.len() as u32;
		let bytes = len.to_ne_bytes();
		call.extend_from_slice(&bytes);

		let result = self
			.rpc_state_call("TransactionPaymentCallApi_query_call_info", &call, at)
			.await?;
		let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		let result = RuntimeDispatchInfo::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

		Ok(result)
	}

	pub async fn api_transaction_payment_query_call_fee_details(
		&self,
		mut call: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, ClientError> {
		let len = call.len() as u32;
		let bytes = len.to_ne_bytes();
		call.extend_from_slice(&bytes);

		let result = self
			.rpc_state_call("TransactionPaymentCallApi_query_call_fee_details", &call, at)
			.await?;
		let result = hex::decode(result.trim_start_matches("0x")).map_err(|e| e.to_string())?;
		let result = FeeDetails::decode(&mut result.as_slice()).map_err(|e| e.to_string())?;

		Ok(result)
	}
}
