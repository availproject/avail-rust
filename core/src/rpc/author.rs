use crate::error::Error;
use primitive_types::H256;
use std::array::TryFromSliceError;
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
	let keys = SessionKeys::try_from(value.as_slice())?;
	Ok(keys)
}

pub async fn submit_extrinsic(client: &RpcClient, extrinsic: &[u8]) -> Result<H256, subxt_rpcs::Error> {
	let ext = std::format!("0x{}", const_hex::encode(extrinsic));
	let params = rpc_params![ext];
	let value: H256 = client.request("author_submitExtrinsic", params).await?;
	Ok(value)
}
