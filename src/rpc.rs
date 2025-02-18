use crate::{
	avail::runtime_types::{da_runtime::primitives::SessionKeys, frame_system::limits::BlockLength},
	from_substrate::{NodeRole, PeerInfo, SyncState},
	utils, ABlockDetailsRPC, AvailHeader, BlockNumber, Cell, Client, GDataProof, GRow, H256,
};
use avail_core::data_proof::ProofResponse;

use kate_recovery::matrix::Position;
use poly_multiproof::method1::{M1NoPrecomp, Proof};
use poly_multiproof::msm::blst::BlstMSMEngine;
use poly_multiproof::traits::PolyMultiProofNoPrecomp;

use subxt::{
	backend::legacy::rpc_methods::{Bytes, RuntimeVersion, SystemHealth},
	rpc_params,
};

/// Arbitrary properties defined in chain spec as a JSON object
pub type SystemProperties = serde_json::map::Map<String, serde_json::Value>;

pub mod system {
	use super::*;

	pub async fn account_next_index(client: &Client, account: String) -> Result<u32, subxt::Error> {
		let params = rpc_params![account];
		let value = client.rpc_client.request("system_accountNextIndex", params).await?;
		Ok(value)
	}

	pub async fn chain(client: &Client) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_chain", params).await?;
		Ok(value)
	}

	pub async fn chain_type(client: &Client) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_chainType", params).await?;
		Ok(value)
	}

	pub async fn health(client: &Client) -> Result<SystemHealth, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_health", params).await?;
		Ok(value)
	}

	pub async fn local_listen_addresses(client: &Client) -> Result<Vec<String>, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_localListenAddresses", params).await?;
		Ok(value)
	}

	pub async fn local_peer_id(client: &Client) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_localPeerId", params).await?;
		Ok(value)
	}

	pub async fn name(client: &Client) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_name", params).await?;
		Ok(value)
	}

	pub async fn node_roles(client: &Client) -> Result<Vec<NodeRole>, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_nodeRoles", params).await?;
		Ok(value)
	}

	pub async fn peers(client: &Client) -> Result<Vec<PeerInfo>, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_peers", params).await?;
		Ok(value)
	}

	pub async fn properties(client: &Client) -> Result<SystemProperties, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_properties", params).await?;
		Ok(value)
	}

	pub async fn sync_state(client: &Client) -> Result<SyncState, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_syncState", params).await?;
		Ok(value)
	}

	pub async fn version(client: &Client) -> Result<String, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("system_version", params).await?;
		Ok(value)
	}
}

pub mod chain {
	use super::*;

	pub async fn get_block(client: &Client, at: Option<H256>) -> Result<ABlockDetailsRPC, subxt::Error> {
		let params = rpc_params![at];
		let value = client.rpc_client.request("chain_getBlock", params).await?;
		Ok(value)
	}

	pub async fn get_block_hash(client: &Client, block_number: Option<BlockNumber>) -> Result<H256, subxt::Error> {
		let params = rpc_params![block_number];
		let value = client.rpc_client.request("chain_getBlockHash", params).await?;
		Ok(value)
	}

	pub async fn get_finalized_head(client: &Client) -> Result<H256, subxt::Error> {
		let params = rpc_params![];
		let value = client.rpc_client.request("chain_getFinalizedHead", params).await?;
		Ok(value)
	}

	pub async fn get_header(client: &Client, at: Option<H256>) -> Result<AvailHeader, subxt::Error> {
		let params = rpc_params![at];
		let value = client.rpc_client.request("chain_getHeader", params).await?;
		Ok(value)
	}
}

pub mod author {
	use super::*;

	pub async fn rotate_keys(client: &Client) -> Result<SessionKeys, subxt::Error> {
		let params = rpc_params![];
		let value: Bytes = client.rpc_client.request("author_rotateKeys", params).await?;
		let keys = utils::deconstruct_session_keys(value.0)?;
		Ok(keys)
	}

	pub async fn submit_extrinsic(client: &Client, extrinsic: &[u8]) -> Result<H256, subxt::Error> {
		let ext = std::format!("0x{}", hex::encode(extrinsic));
		let params = rpc_params![ext];
		let value: String = client.rpc_client.request("author_submitExtrinsic", params).await?;
		let value = utils::hex_string_to_h256(&value)?;
		Ok(value)
	}
}

pub mod state {
	use super::*;

	pub async fn get_runtime_version(client: &Client, at: Option<H256>) -> Result<RuntimeVersion, subxt::Error> {
		let params = rpc_params![at];
		let value = client.rpc_client.request("state_getRuntimeVersion", params).await?;
		Ok(value)
	}

	pub async fn call(client: &Client, method: &str, data: &[u8], at: Option<H256>) -> Result<String, subxt::Error> {
		let data = std::format!("0x{}", hex::encode(data));
		let params = rpc_params![method, data, at];
		let value = client.rpc_client.request("state_call", params).await?;
		Ok(value)
	}
}

pub mod kate {
	use ::kate::{
		gridgen::{domain_points, multiproof_block, AsBytes, Commitment},
		ArkScalar,
	};

	use kate_recovery::matrix::{Dimensions, Partition, Position};
	use log::error;
	use poly_multiproof::{ark_bls12_381::Bls12_381, merlin::Transcript};

	use subxt::ext::futures::future::join_all;
use subxt_signer::bip39::rand::thread_rng;

	use crate::{
		primitives::kate::{Cells, GProof, GRawScalar},
		utils::extract_kate,
	};

	use super::*;

	pub async fn block_length(client: &Client, at: Option<H256>) -> Result<BlockLength, subxt::Error> {
		let params = rpc_params![at];
		let value = client.rpc_client.request("kate_blockLength", params).await?;
		Ok(value)
	}

	pub async fn query_data_proof(
		client: &Client,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, subxt::Error> {
		let params = rpc_params![transaction_index, at];
		let value = client.rpc_client.request("kate_queryDataProof", params).await?;
		Ok(value)
	}

	pub async fn query_proof(
		client: &Client,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<GDataProof>, subxt::Error> {
		let params = rpc_params![cells, at];
		let value = client.rpc_client.request("kate_queryProof", params).await?;
		Ok(value)
	}

	pub async fn query_multi_proof(
		client: &RpcClient,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<GMultiProof>, ClientError> {
		let params = rpc_params![cells, at];
		let value = client.request("kate_queryMultiProof", params).await?;
		Ok(value)
	}

	pub async fn query_multi_proof_using_hash(
		client: &RpcClient,
		at: Option<H256>,
		block_matrix_partition: Partition,
	) -> Result<(Vec<GMultiProof>, u16, u16, Vec<u8>), ClientError> {
		let header = chain::get_header(client, at).await?;

		let Some((rows, cols, _, commitment)) = extract_kate(&header.extension) else {
			return Err(ClientError::Custom(
				"Skipping block without header extension".to_string(),
			));
		};
		let Some(dimensions) = Dimensions::new(rows, cols) else {
			return Err(ClientError::Custom(
				"Skipping block with invalid dimensions".to_string(),
			));
		};

		if dimensions.cols().get() <= 2 {
			error!("More than 2 columns are required");
			return Ok((vec![], rows, cols, commitment));
		}

		let positions: Vec<Position> = dimensions
			.iter_extended_partition_positions(&block_matrix_partition)
			.collect();

		let create_cell = |positions: &&[Position]| create_cell((*positions).to_vec());
		let rpc_batches = positions.chunks(30).collect::<Vec<_>>();
		let parallel_batches = rpc_batches
			.chunks(8)
			.map(|batch| join_all(batch.iter().map(create_cell)));
	
		let mut cells = vec![];
		for batch in parallel_batches {
			for (i, result) in batch.await.into_iter().enumerate() {
				cells.append(&mut result.unwrap().to_vec());
			}
		}
		
		let proofs: Vec<(Vec<GRawScalar>, GProof)> = query_multi_proof(client, cells.to_vec(), at)
			.await
			.map_err(|error| ClientError::Custom(format!("{:?}", error)))?;

		Ok((proofs, rows, cols, commitment))
	}

	pub async fn verify_multi_proof_using_hash(
		proof: Vec<GMultiProof>,
		rows: u16,
		cols: u16,
		commitments: Vec<u8>,
	) -> Result<bool, ClientError> {
		type E = Bls12_381;
		type M = BlstMSMEngine;
		let pmp = M1NoPrecomp::<E, M>::new(256, 256, &mut thread_rng());

		let target_dims = Dimensions::new_from(16, 64).unwrap();
		let dimensions = Dimensions::new(rows, cols).unwrap();
		let mp_block = multiproof_block(0, 0, dimensions, target_dims).unwrap();
		let commits = commitments
			.chunks_exact(48)
			.skip(cols as usize)
			.take(rows as usize)
			.map(|c| Commitment::from_bytes(c.try_into().unwrap()))
			.collect::<Result<Vec<_>, _>>()
			.unwrap();

		let block_commits = &commits[mp_block.start_x..mp_block.end_x];

		for (eval, proof) in proof.iter() {
			let evals_flat = eval
				.into_iter()
				.map(|e| ArkScalar::from_bytes(&e.to_little_endian()))
				.collect::<Result<Vec<_>, _>>()
				.unwrap();
			let evals_grid = evals_flat.chunks_exact(cols as usize).collect::<Vec<_>>();
			let points = domain_points(256)
				.map_err(|_| ClientError::Custom("Failed to generate domain points".to_string()))?;
			let proofs = Proof::from_bytes(&proof.0).unwrap();
			let verified = pmp
				.verify(
					&mut Transcript::new(b"avail-mp"),
					&block_commits,
					&points[mp_block.start_x..mp_block.end_x],
					&evals_grid,
					&proofs,
				)
				.map_err(|_| ClientError::Custom("Failed to verify proof".to_string()))?;
			if !verified {
				return Ok(false);
			}
		}

		Ok(true)
	}

	pub async fn verify_multi_proof(
		client: &RpcClient,
		proof: Vec<GMultiProof>,
		at: Option<H256>,
	) -> Result<bool, ClientError> {
		let params = rpc_params![proof, at];
		let value = client.request("kate_verifyProof", params).await?;
		Ok(value)
	}

	pub async fn query_rows(
		client: &RpcClient,
		rows: Vec<u32>,
		at: Option<H256>,
	) -> Result<Vec<GRow>, ClientError> {
		let params = rpc_params![rows, at];
		let value = client.rpc_client.request("kate_queryRows", params).await?;
		Ok(value)
	}

	async fn create_cell(positions: Vec<Position>) -> Result<Cells, ClientError>{
		let cells: Cells = positions
			.iter()
			.map(|p| Cell {
				row: p.row,
				col: p.col as u32,
			})
			.collect::<Vec<_>>()
			.try_into()
			.map_err(|_| ClientError::Custom("Failed to convert to cells".to_string()))?;
	
		Ok(cells)
	}
}
