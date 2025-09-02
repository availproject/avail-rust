use crate::HashNumber;
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use subxt_rpcs::{RpcClient, rpc_params};

pub async fn fetch_extrinsics_v1(
	client: &RpcClient,
	block_id: HashNumber,
	options: Options,
) -> Result<Vec<ExtrinsicInfo>, subxt_rpcs::Error> {
	let options: RpcOptions = RpcOptions {
		filter: Filter {
			transaction: Some(options.transaction_filter),
			signature: SignatureFilter {
				ss58_address: options.ss58_address,
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
	pub transaction_filter: TransactionFilter,
	pub ss58_address: Option<String>,
	pub app_id: Option<u32>,
	pub nonce: Option<u32>,
	pub encode_as: EncodeSelector,
}

#[derive(Clone, Default, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum EncodeSelector {
	None = 0,
	#[default]
	Call = 1,
	Extrinsic = 2,
}

#[derive(Debug, Clone)]
pub struct ExtrinsicInfo {
	// Hex string encoded
	pub data: Option<String>,
	pub tx_hash: H256,
	pub tx_index: u32,
	pub pallet_id: u8,
	pub variant_id: u8,
	pub signature: Option<TransactionSignature>,
}

impl From<ExtrinsicInformation> for ExtrinsicInfo {
	fn from(value: ExtrinsicInformation) -> Self {
		Self {
			data: value.encoded,
			tx_hash: value.tx_hash,
			tx_index: value.tx_index,
			pallet_id: value.pallet_id,
			variant_id: value.call_id,
			signature: value.signature,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSignature {
	pub ss58_address: Option<String>,
	pub nonce: u32,
	pub app_id: u32,
	pub mortality: Option<(u64, u64)>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TransactionFilter {
	All,
	TxHash(Vec<H256>),
	TxIndex(Vec<u32>),
	Pallet(Vec<u8>),
	PalletCall(Vec<(u8, u8)>),
}

impl TransactionFilter {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Default for TransactionFilter {
	fn default() -> Self {
		Self::All
	}
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct RpcOptions {
	pub filter: Filter,
	pub encode_selector: Option<EncodeSelector>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct Filter {
	pub transaction: Option<TransactionFilter>,
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
	pub signature: Option<TransactionSignature>,
}
