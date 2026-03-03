use super::Error;
use primitive_types::H256;
use serde::Deserialize;
use std::array::TryFromSliceError;
use std::str::FromStr;
use subxt_rpcs::{RpcClient, rpc_params};

#[derive(Debug, Clone)]
pub struct SessionKeys {
	pub babe: [u8; 32],
	pub grandpa: [u8; 32],
	pub im_online: [u8; 32],
	pub authority_discovery: [u8; 32],
}

impl TryFrom<&[u8]> for SessionKeys {
	type Error = String;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		if value.len() != 128 {
			return Err(String::from("Session keys len cannot have length be more or less than 128"));
		}

		let err = |e: TryFromSliceError| e.to_string();

		let babe: [u8; 32] = value[0..32].try_into().map_err(err)?;
		let grandpa: [u8; 32] = value[32..64].try_into().map_err(err)?;
		let im_online: [u8; 32] = value[64..96].try_into().map_err(err)?;
		let authority_discovery: [u8; 32] = value[96..128].try_into().map_err(err)?;
		Ok(Self { babe, grandpa, im_online, authority_discovery })
	}
}

pub async fn rotate_keys(client: &RpcClient) -> Result<SessionKeys, Error> {
	let params = rpc_params![];
	let value: Vec<u8> = client.request("author_rotateKeys", params).await?;
	let keys = SessionKeys::try_from(value.as_slice()).map_err(|e| Error::MalformedResponse(e.to_string()))?;
	Ok(keys)
}

pub async fn submit_extrinsic(client: &RpcClient, extrinsic: &[u8]) -> Result<H256, Error> {
	let ext = std::format!("0x{}", const_hex::encode(extrinsic));
	let params = rpc_params![ext];
	let value: H256Compat = client.request("author_submitExtrinsic", params).await?;
	value.into_h256()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum H256Compat {
	Hex(String),
	Bytes(Vec<u8>),
}

impl H256Compat {
	fn into_h256(self) -> Result<H256, Error> {
		match self {
			H256Compat::Hex(value) => H256::from_str(value.as_str()).map_err(|e| Error::MalformedResponse(e.to_string())),
			H256Compat::Bytes(value) => {
				if value.len() != 32 {
					return Err(Error::MalformedResponse("Expected exactly 32 bytes for H256".into()));
				}
				Ok(H256::from_slice(value.as_slice()))
			},
		}
	}
}
