use crate::platform::sleep;
use avail_rust_core::{
	AvailHeader, H256, HashNumber,
	ext::{
		codec::Decode,
		subxt_rpcs::{
			client::RpcParams,
			methods::legacy::{RuntimeVersion, SystemHealth},
		},
	},
	grandpa::GrandpaJustification,
	rpc::{
		self, BlockWithJustifications,
		author::SessionKeys,
		kate::{BlockLength, Cell, GDataProof, GRow, ProofResponse},
		rpc_methods::RpcMethods,
		system::{NodeRole, PeerInfo, SyncState, SystemProperties, fetch_events_v1_types, fetch_extrinsics_v1_types},
	},
};
use std::time::Duration;

use crate::Client;

#[derive(Clone)]
pub struct RpcAPI {
	client: Client,
}

impl RpcAPI {
	pub fn new(client: Client) -> Self {
		Self { client }
	}

	pub async fn call<T: serde::de::DeserializeOwned>(
		&self,
		method: &str,
		params: RpcParams,
	) -> Result<T, avail_rust_core::Error> {
		Ok(rpc::call_raw(&self.client.rpc_client, method, params).await?)
	}

	pub async fn system_account_next_index(&self, address: &str) -> Result<u32, avail_rust_core::Error> {
		Ok(rpc::system::account_next_index(&self.client.rpc_client, address).await?)
	}

	pub async fn system_chain(&self) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::system::chain(&self.client.rpc_client).await?)
	}

	pub async fn system_chain_type(&self) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::system::chain_type(&self.client.rpc_client).await?)
	}

	pub async fn system_health(&self) -> Result<SystemHealth, avail_rust_core::Error> {
		Ok(rpc::system::health(&self.client.rpc_client).await?)
	}

	pub async fn system_local_listen_addresses(&self) -> Result<Vec<String>, avail_rust_core::Error> {
		Ok(rpc::system::local_listen_addresses(&self.client.rpc_client).await?)
	}

	pub async fn system_local_peer_id(&self) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::system::local_peer_id(&self.client.rpc_client).await?)
	}

	pub async fn system_name(&self) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::system::name(&self.client.rpc_client).await?)
	}

	pub async fn system_node_roles(&self) -> Result<Vec<NodeRole>, avail_rust_core::Error> {
		Ok(rpc::system::node_roles(&self.client.rpc_client).await?)
	}

	pub async fn system_peers(&self) -> Result<Vec<PeerInfo>, avail_rust_core::Error> {
		Ok(rpc::system::peers(&self.client.rpc_client).await?)
	}

	pub async fn system_properties(&self) -> Result<SystemProperties, avail_rust_core::Error> {
		Ok(rpc::system::properties(&self.client.rpc_client).await?)
	}

	pub async fn system_sync_state(&self) -> Result<SyncState, avail_rust_core::Error> {
		Ok(rpc::system::sync_state(&self.client.rpc_client).await?)
	}

	pub async fn system_version(&self) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::system::version(&self.client.rpc_client).await?)
	}

	pub async fn chain_get_block(
		&self,
		at: Option<H256>,
	) -> Result<Option<BlockWithJustifications>, avail_rust_core::Error> {
		Ok(rpc::chain::get_block(&self.client.rpc_client, at).await?)
	}

	pub async fn chain_get_block_hash(
		&self,
		block_height: Option<u32>,
	) -> Result<Option<H256>, avail_rust_core::Error> {
		Ok(rpc::chain::get_block_hash(&self.client.rpc_client, block_height).await?)
	}

	pub async fn chain_get_finalized_head(&self) -> Result<H256, avail_rust_core::Error> {
		Ok(rpc::chain::get_finalized_head(&self.client.rpc_client).await?)
	}

	pub async fn chain_get_header(&self, at: Option<H256>) -> Result<Option<AvailHeader>, avail_rust_core::Error> {
		Ok(rpc::chain::get_header(&self.client.rpc_client, at).await?)
	}

	pub async fn author_rotate_keys(&self) -> Result<SessionKeys, avail_rust_core::Error> {
		rpc::author::rotate_keys(&self.client.rpc_client).await
	}

	pub async fn author_submit_extrinsic(&self, extrinsic: &[u8]) -> Result<H256, avail_rust_core::Error> {
		Ok(rpc::author::submit_extrinsic(&self.client.rpc_client, extrinsic).await?)
	}

	pub async fn state_get_runtime_version(&self, at: Option<H256>) -> Result<RuntimeVersion, avail_rust_core::Error> {
		Ok(rpc::state::get_runtime_version(&self.client.rpc_client, at).await?)
	}

	pub async fn state_call(
		&self,
		method: &str,
		data: &[u8],
		at: Option<H256>,
	) -> Result<String, avail_rust_core::Error> {
		Ok(rpc::state::call(&self.client.rpc_client, method, data, at).await?)
	}

	pub async fn state_get_metadata(&self, at: Option<H256>) -> Result<Vec<u8>, avail_rust_core::Error> {
		rpc::state::get_metadata(&self.client.rpc_client, at).await
	}

	pub async fn state_get_storage(
		&self,
		key: &str,
		at: Option<H256>,
	) -> Result<Option<Vec<u8>>, avail_rust_core::Error> {
		rpc::state::get_storage(&self.client.rpc_client, key, at).await
	}

	pub async fn state_get_keys_paged(
		&self,
		prefix: Option<String>,
		count: u32,
		start_key: Option<String>,
		at: Option<H256>,
	) -> Result<Vec<String>, avail_rust_core::Error> {
		rpc::state::get_keys_paged(&self.client.rpc_client, prefix, count, start_key, at).await
	}

	pub async fn rpc_methods(&self) -> Result<RpcMethods, avail_rust_core::Error> {
		Ok(rpc::rpc_methods::call(&self.client.rpc_client).await?)
	}

	pub async fn chainspec_v1_genesishash(&self) -> Result<H256, avail_rust_core::Error> {
		rpc::chainspec::v1_genesishash(&self.client.rpc_client).await
	}

	pub async fn kate_block_length(&self, at: Option<H256>) -> Result<BlockLength, avail_rust_core::Error> {
		Ok(rpc::kate::block_length(&self.client.rpc_client, at).await?)
	}

	pub async fn kate_query_data_proof(
		&self,
		transaction_index: u32,
		at: Option<H256>,
	) -> Result<ProofResponse, avail_rust_core::Error> {
		Ok(rpc::kate::query_data_proof(&self.client.rpc_client, transaction_index, at).await?)
	}

	pub async fn kate_query_proof(
		&self,
		cells: Vec<Cell>,
		at: Option<H256>,
	) -> Result<Vec<GDataProof>, avail_rust_core::Error> {
		Ok(rpc::kate::query_proof(&self.client.rpc_client, cells, at).await?)
	}

	pub async fn kate_query_rows(&self, rows: Vec<u32>, at: Option<H256>) -> Result<Vec<GRow>, avail_rust_core::Error> {
		Ok(rpc::kate::query_rows(&self.client.rpc_client, rows, at).await?)
	}

	pub async fn grandpa_block_justification(
		&self,
		at: u32,
	) -> Result<Option<GrandpaJustification>, avail_rust_core::Error> {
		let result = rpc::grandpa::block_justification(&self.client.rpc_client, at).await?;
		let Some(result) = result else { return Ok(None) };

		let justification = const_hex::decode(result.trim_start_matches("0x"))
			.map_err(|x| avail_rust_core::Error::from(x.to_string()))?;

		let justification = GrandpaJustification::decode(&mut justification.as_slice()).map_err(|e| e.to_string())?;
		Ok(Some(justification))
	}

	pub async fn system_fetch_events_v1(
		&self,
		at: H256,
		options: Option<fetch_events_v1_types::Options>,
	) -> Result<fetch_events_v1_types::Output, avail_rust_core::Error> {
		Ok(rpc::system::fetch_events_v1(&self.client.rpc_client, at, options).await?)
	}

	pub async fn system_fetch_events_v1_with_retries(
		&self,
		at: H256,
		options: Option<fetch_events_v1_types::Options>,
	) -> Result<fetch_events_v1_types::Output, avail_rust_core::Error> {
		let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
		loop {
			let events = rpc::system::fetch_events_v1(&self.client.rpc_client, at, options.clone()).await;
			let events = match events {
				Ok(x) => x,
				Err(err) => {
					let Some(duration) = sleep_duration.pop() else {
						return Err(err.into());
					};

					#[cfg(feature = "tracing")]
					trace_warn(&std::format!(
						"Fetching events (system_fetchEventsV1) ended with Err {}. Sleep for {} seconds",
						err.to_string(),
						duration
					));
					sleep(Duration::from_secs(duration)).await;
					continue;
				},
			};

			return Ok(events);
		}
	}

	pub async fn system_fetch_extrinsics_v1(
		&self,
		block_id: HashNumber,
		options: Option<fetch_extrinsics_v1_types::Options>,
	) -> Result<fetch_extrinsics_v1_types::Output, avail_rust_core::Error> {
		Ok(rpc::system::fetch_extrinsics_v1(&self.client.rpc_client, block_id, options).await?)
	}

	pub async fn system_fetch_extrinsics_v1_with_retries(
		&self,
		block_id: HashNumber,
		options: Option<fetch_extrinsics_v1_types::Options>,
	) -> Result<fetch_extrinsics_v1_types::Output, avail_rust_core::Error> {
		let mut sleep_duration: Vec<u64> = vec![8, 5, 3, 2, 1];
		loop {
			let txs = rpc::system::fetch_extrinsics_v1(&self.client.rpc_client, block_id, options.clone()).await;
			let txs = match txs {
				Ok(x) => x,
				Err(err) => {
					let Some(duration) = sleep_duration.pop() else {
						return Err(err.into());
					};

					#[cfg(feature = "tracing")]
					trace_warn(&std::format!(
						"Fetching extrinsics (system_fetchExtrinsicsV1) ended with Err {}. Sleep for {} seconds",
						err.to_string(),
						duration
					));
					sleep(Duration::from_secs(duration)).await;
					continue;
				},
			};

			return Ok(txs);
		}
	}
}

#[cfg(feature = "tracing")]
fn trace_warn(message: &str) {
	tracing::warn!(target: "lib", message);
}
