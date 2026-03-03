use std::str::FromStr;

use crate::{AccountId, HashNumber, rpc::Error};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use subxt_rpcs::{RpcClient, rpc_params};

pub async fn fetch_extrinsics(
	client: &RpcClient,
	at: HashNumber,
	allow_list: Option<Vec<AllowedExtrinsic>>,
	sig_filter: SignatureFilter,
	data_format: DataFormat,
) -> Result<Vec<Extrinsic>, Error> {
	let params = rpc_params![at, allow_list, sig_filter, data_format];
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
	pub ss58_address: Option<String>,
	pub app_id: Option<u32>,
	pub nonce: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Extrinsic {
	pub data: String,
	pub ext_hash: H256,
	pub ext_index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub signature: Option<TransactionSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSignature {
	pub account_id: Option<AccountId>,
	pub nonce: u32,
}
