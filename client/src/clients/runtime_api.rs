use client_core::{
	ext::codec,
	from_substrate::{FeeDetails, RuntimeDispatchInfo},
	Error, H256,
};

use crate::Client;

#[derive(Clone)]
pub struct RuntimeApi {
	client: Client,
}

impl RuntimeApi {
	pub fn new(client: Client) -> Self {
		Self { client }
	}

	pub async fn call<T: codec::Decode>(&self, method: &str, data: &[u8], at: Option<H256>) -> Result<T, Error> {
		client_core::runtime_api::call_raw(&self.client.rpc_client, method, data, at).await
	}

	pub async fn transaction_payment_query_info(
		&self,
		extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, Error> {
		client_core::runtime_api::api_transaction_payment_query_info(&self.client.rpc_client, extrinsic, at).await
	}

	pub async fn transaction_payment_query_fee_details(
		&self,
		extrinsic: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, Error> {
		client_core::runtime_api::api_transaction_payment_query_fee_details(&self.client.rpc_client, extrinsic, at)
			.await
	}

	pub async fn transaction_payment_query_call_info(
		&self,
		call: Vec<u8>,
		at: Option<H256>,
	) -> Result<RuntimeDispatchInfo, Error> {
		client_core::runtime_api::api_transaction_payment_query_call_info(&self.client.rpc_client, call, at).await
	}

	pub async fn transaction_payment_query_call_fee_details(
		&self,
		call: Vec<u8>,
		at: Option<H256>,
	) -> Result<FeeDetails, Error> {
		client_core::runtime_api::api_transaction_payment_query_call_fee_details(&self.client.rpc_client, call, at)
			.await
	}
}
