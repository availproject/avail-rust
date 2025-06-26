use crate::config::TransactionLocation;

use super::AvailHeader;
use primitive_types::H256;
use serde::{Deserialize, Deserializer};
use subxt_core::config::{substrate::BlakeTwo256, Hasher};
use subxt_rpcs::{rpc_params, RpcClient};

/// The response from `chain_getBlock`
#[derive(Debug, Clone, Deserialize)]
pub struct BlockWithJustifications {
	/// The block itself.
	pub block: Block,
	/// Block justification.
	pub justifications: Option<Vec<BlockJustification>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Block {
	/// The block header.
	pub header: AvailHeader,
	#[serde(deserialize_with = "from_string_to_vec")]
	pub extrinsics: Vec<Vec<u8>>,
}

impl Block {
	pub fn has_transaction(&self, tx_hash: H256) -> Option<TransactionLocation> {
		for (i, tx) in self.extrinsics.iter().enumerate() {
			if BlakeTwo256::hash(tx) == tx_hash {
				return Some(TransactionLocation::from((tx_hash, i as u32)));
			}
		}

		None
	}
}

fn from_string_to_vec<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
where
	D: Deserializer<'de>,
{
	let buf = Vec::<String>::deserialize(deserializer)?;
	let result: Result<Vec<Vec<u8>>, _> = buf
		.into_iter()
		.map(|x| hex::decode(x.trim_start_matches("0x")))
		.collect();
	match result {
		Ok(res) => Ok(res),
		Err(err) => Err(serde::de::Error::custom(err)),
	}
}

/// An abstraction over justification for a block's validity under a consensus algorithm.
pub type BlockJustification = (ConsensusEngineId, EncodedJustification);
/// Consensus engine unique ID.
pub type ConsensusEngineId = [u8; 4];
/// The encoded justification specific to a consensus engine.
pub type EncodedJustification = Vec<u8>;

pub async fn get_block(
	client: &RpcClient,
	at: Option<H256>,
) -> Result<Option<BlockWithJustifications>, subxt_rpcs::Error> {
	let params = rpc_params![at];
	let res: Option<BlockWithJustifications> = client.request("chain_getBlock", params).await?;
	let Some(value) = res else { return Ok(None) };
	Ok(Some(value))
}

pub async fn get_block_hash(client: &RpcClient, block_height: Option<u32>) -> Result<Option<H256>, subxt_rpcs::Error> {
	let params = rpc_params![block_height];
	let value = client.request("chain_getBlockHash", params).await?;
	Ok(value)
}

pub async fn get_header(client: &RpcClient, at: Option<H256>) -> Result<Option<AvailHeader>, subxt_rpcs::Error> {
	let params = rpc_params![at];
	let value = client.request("chain_getHeader", params).await?;
	Ok(value)
}

pub async fn get_finalized_head(client: &RpcClient) -> Result<H256, subxt_rpcs::Error> {
	let value = client.request("chain_getFinalizedHead", rpc_params![]).await?;
	Ok(value)
}
