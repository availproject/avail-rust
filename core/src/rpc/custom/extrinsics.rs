use crate::{
	AccountId,
	rpc::{Error, custom::BlockId},
};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use subxt_rpcs::{RpcClient, rpc_params};

#[derive(Default, Debug, Clone)]
pub struct FetchParams {
	pub allow_list: Option<Vec<AllowedExtrinsic>>,
	pub sig_filter: SignatureFilter,
	pub data_format: DataFormat,
}

impl FetchParams {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_allow_list(mut self, value: Vec<AllowedExtrinsic>) -> Self {
		self.allow_list = Some(value);
		self
	}

	pub fn with_sig_filter(mut self, value: SignatureFilter) -> Self {
		self.sig_filter = value;
		self
	}

	pub fn with_data_format(mut self, value: DataFormat) -> Self {
		self.data_format = value;
		self
	}
}

impl From<Option<Vec<AllowedExtrinsic>>> for FetchParams {
	fn from(value: Option<Vec<AllowedExtrinsic>>) -> Self {
		Self { allow_list: value, ..Default::default() }
	}
}

impl From<Vec<AllowedExtrinsic>> for FetchParams {
	fn from(value: Vec<AllowedExtrinsic>) -> Self {
		Self::from(Some(value))
	}
}

pub async fn fetch_extrinsics(client: &RpcClient, at: BlockId, params: FetchParams) -> Result<Vec<Extrinsic>, Error> {
	let params = rpc_params![at, params.allow_list, params.sig_filter, params.data_format];
	let value: Vec<Extrinsic> = client.request("custom_extrinsics", params).await?;
	Ok(value)
}

#[derive(Debug, Clone, Default, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum DataFormat {
	None = 0,
	Call = 1,
	#[default]
	Extrinsic = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedExtrinsic {
	TxHash(H256),
	TxIndex(u32),
	Pallet(u8),
	PalletCall((u8, u8)),
}

impl TryFrom<&str> for AllowedExtrinsic {
	type Error = String;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let hash = H256::from_str(value).map_err(|e| e.to_string())?;
		Ok(Self::TxHash(hash))
	}
}

impl From<H256> for AllowedExtrinsic {
	fn from(value: H256) -> Self {
		Self::TxHash(value)
	}
}

impl From<u32> for AllowedExtrinsic {
	fn from(value: u32) -> Self {
		Self::TxIndex(value)
	}
}

impl From<u8> for AllowedExtrinsic {
	fn from(value: u8) -> Self {
		Self::Pallet(value)
	}
}

impl From<(u8, u8)> for AllowedExtrinsic {
	fn from(value: (u8, u8)) -> Self {
		Self::PalletCall(value)
	}
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SignatureFilter {
	pub account_id: Option<String>,
	pub nonce: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Extrinsic {
	pub data: String,
	pub ext_hash: H256,
	pub ext_index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub account_id: Option<AccountId>,
	pub nonce: Option<u32>,
}
