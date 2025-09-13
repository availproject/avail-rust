use std::str::FromStr;

use crate::HashNumber;
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use subxt_rpcs::{RpcClient, rpc_params};

pub async fn fetch_extrinsics_v1(
	client: &RpcClient,
	block_id: HashNumber,
	options: &Options,
) -> Result<Vec<ExtrinsicInfo>, subxt_rpcs::Error> {
	let options: RpcOptions = RpcOptions {
		filter: Filter {
			transaction: Some(options.transaction_filter.clone()),
			signature: SignatureFilter {
				ss58_address: options.ss58_address.clone(),
				app_id: options.app_id,
				nonce: options.nonce,
			},
		},
		encode_selector: Some(options.encode_as),
	};
	let params = rpc_params![block_id, options];
	let value: Vec<ExtrinsicInformation> = client.request("system_fetchExtrinsicsV1", params).await?;
	let value: Vec<ExtrinsicInfo> = value.into_iter().map(|x| x.into()).collect();
	Ok(value)
}

#[derive(Clone, Default)]
pub struct Options {
	pub transaction_filter: ExtrinsicFilter,
	pub ss58_address: Option<String>,
	pub app_id: Option<u32>,
	pub nonce: Option<u32>,
	pub encode_as: EncodeSelector,
}

#[derive(Debug, Clone, Default, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum EncodeSelector {
	None = 0,
	Call = 1,
	#[default]
	Extrinsic = 2,
}

#[derive(Debug, Clone)]
pub struct ExtrinsicInfo {
	// Hex string encoded
	pub data: Option<String>,
	pub ext_hash: H256,
	pub ext_index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub signer_payload: Option<SignerPayload>,
}

impl From<ExtrinsicInformation> for ExtrinsicInfo {
	fn from(value: ExtrinsicInformation) -> Self {
		Self {
			data: value.encoded,
			ext_hash: value.tx_hash,
			ext_index: value.tx_index,
			pallet_id: value.pallet_id,
			variant_id: value.call_id,
			signer_payload: value.signature,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerPayload {
	pub ss58_address: Option<String>,
	pub nonce: u32,
	pub app_id: u32,
	pub mortality: Option<(u64, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtrinsicFilter {
	All,
	TxHash(Vec<H256>),
	TxIndex(Vec<u32>),
	Pallet(Vec<u8>),
	PalletCall(Vec<(u8, u8)>),
}

impl ExtrinsicFilter {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Default for ExtrinsicFilter {
	fn default() -> Self {
		Self::All
	}
}

impl TryFrom<String> for ExtrinsicFilter {
	type Error = crate::Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl TryFrom<&String> for ExtrinsicFilter {
	type Error = crate::Error;

	fn try_from(value: &String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
	}
}

impl TryFrom<&str> for ExtrinsicFilter {
	type Error = crate::Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let h256 = H256::from_str(value).map_err(|x| x.to_string())?;
		Ok(Self::TxHash(vec![h256]))
	}
}

impl From<H256> for ExtrinsicFilter {
	fn from(value: H256) -> Self {
		Self::TxHash(vec![value])
	}
}

impl From<u32> for ExtrinsicFilter {
	fn from(value: u32) -> Self {
		Self::TxIndex(vec![value])
	}
}

impl From<u8> for ExtrinsicFilter {
	fn from(value: u8) -> Self {
		Self::Pallet(vec![value])
	}
}

impl From<(u8, u8)> for ExtrinsicFilter {
	fn from(value: (u8, u8)) -> Self {
		Self::PalletCall(vec![value])
	}
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct RpcOptions {
	pub filter: Filter,
	pub encode_selector: Option<EncodeSelector>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct Filter {
	pub transaction: Option<ExtrinsicFilter>,
	pub signature: SignatureFilter,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct SignatureFilter {
	pub ss58_address: Option<String>,
	pub app_id: Option<u32>,
	pub nonce: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtrinsicInformation {
	// Hex string encoded
	pub encoded: Option<String>,
	pub tx_hash: H256,
	pub tx_index: u32,
	pub pallet_id: u8,
	pub call_id: u8,
	pub signature: Option<SignerPayload>,
}
