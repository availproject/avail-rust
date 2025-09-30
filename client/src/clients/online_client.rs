//! Thin cached view of chain metadata and runtime versions fetched from an RPC endpoint.

use crate::{subxt_core::Metadata, subxt_rpcs::RpcClient};
use avail_rust_core::{H256, RpcError, ext::codec::Decode, rpc};
use std::sync::{Arc, RwLock};

/// Shared handle holding runtime metadata and version information.
#[derive(Clone)]
pub struct OnlineClient(pub Arc<RwLock<OnlineClientInner>>);

#[derive(Clone)]
pub struct OnlineClientInner {
	genesis_hash: H256,
	spec_version: u32,
	transaction_version: u32,
	metadata: Metadata,
	global_retries: bool,
}

impl OnlineClient {
	/// Fetches metadata, runtime version, and genesis hash from the node to bootstrap the client.
	///
	/// # Errors
	/// Propagates any underlying [`RpcError`] raised while querying the node or decoding metadata.
	pub async fn new(rpc_client: &RpcClient) -> Result<Self, RpcError> {
		let finalized_hash = rpc::chain::get_finalized_head(rpc_client).await?;
		let rpc_metadata = rpc::state::get_metadata(rpc_client, Some(finalized_hash)).await?;
		let genesis_hash = rpc::chainspec::v1_genesishash(rpc_client).await?;
		let runtime_version = rpc::state::get_runtime_version(rpc_client, Some(finalized_hash)).await?;

		let frame_metadata = frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice())
			.map_err(|e| RpcError::DecodingFailed(e.to_string()))?;
		let metadata = Metadata::try_from(frame_metadata).map_err(|e| RpcError::DecodingFailed(e.to_string()))?;
		let inner = OnlineClientInner {
			genesis_hash,
			spec_version: runtime_version.spec_version,
			transaction_version: runtime_version.transaction_version,
			metadata,
			global_retries: true,
		};
		Ok(Self(Arc::new(RwLock::new(inner))))
	}
}

impl OnlineClient {
	/// Returns the cached genesis hash.
	pub fn genesis_hash(&self) -> H256 {
		let lock = self.0.read().expect("Should not be poisoned");
		lock.genesis_hash
	}

	/// Returns the cached runtime spec version.
	pub fn spec_version(&self) -> u32 {
		let lock = self.0.read().expect("Should not be poisoned");
		lock.spec_version
	}

	/// Returns the cached runtime transaction version.
	pub fn transaction_version(&self) -> u32 {
		let lock = self.0.read().expect("Should not be poisoned");
		lock.transaction_version
	}

	/// Returns the cached metadata handle.
	pub fn metadata(&self) -> Metadata {
		let lock = self.0.read().expect("Should not be poisoned");
		lock.metadata.clone()
	}

	/// Updates the cached genesis hash.
	pub fn set_genesis_hash(&self, value: H256) {
		let mut lock = self.0.write().expect("Should not be poisoned");
		lock.genesis_hash = value;
	}

	/// Updates the cached runtime spec version.
	pub fn set_spec_version(&self, value: u32) {
		let mut lock = self.0.write().expect("Should not be poisoned");
		lock.spec_version = value;
	}

	/// Updates the cached runtime transaction version.
	pub fn set_transaction_version(&self, value: u32) {
		let mut lock = self.0.write().expect("Should not be poisoned");
		lock.transaction_version = value;
	}

	/// Replaces the cached metadata object.
	pub fn set_metadata(&self, value: Metadata) {
		let mut lock = self.0.write().expect("Should not be poisoned");
		lock.metadata = value;
	}

	/// Reports whether new RPC helpers should retry by default.
	pub fn is_global_retries_enabled(&self) -> bool {
		self.0.read().map(|x| x.global_retries).unwrap_or(true)
	}

	/// Updates the default retry preference for newly created helpers.
	pub fn set_global_retries_enabled(&self, value: bool) {
		let mut lock = self.0.write().expect("Should not be poisoned");
		lock.global_retries = value;
	}
}
