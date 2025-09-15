use crate::{subxt_core::Metadata, subxt_rpcs::RpcClient};
use avail_rust_core::{H256, RpcError, ext::codec::Decode, rpc};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct OnlineClient {
	pub inner: Arc<RwLock<OnlineClientInner>>,
}

#[derive(Clone)]
pub struct OnlineClientInner {
	pub genesis_hash: H256,
	pub spec_version: u32,
	pub transaction_version: u32,
	pub metadata: Metadata,
}

impl OnlineClient {
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
		};
		Ok(Self { inner: Arc::new(RwLock::new(inner)) })
	}
}

impl OnlineClient {
	pub fn genesis_hash(&self) -> H256 {
		let lock = self.inner.read().expect("Should not be poisoned");
		lock.genesis_hash
	}

	pub fn spec_version(&self) -> u32 {
		let lock = self.inner.read().expect("Should not be poisoned");
		lock.spec_version
	}

	pub fn transaction_version(&self) -> u32 {
		let lock = self.inner.read().expect("Should not be poisoned");
		lock.transaction_version
	}

	pub fn metadata(&self) -> Metadata {
		let lock = self.inner.read().expect("Should not be poisoned");
		lock.metadata.clone()
	}

	pub fn set_genesis_hash(&self, value: H256) {
		let mut lock = self.inner.write().expect("Should not be poisoned");
		lock.genesis_hash = value;
	}

	pub fn set_spec_version(&self, value: u32) {
		let mut lock = self.inner.write().expect("Should not be poisoned");
		lock.spec_version = value;
	}

	pub fn set_transaction_version(&self, value: u32) {
		let mut lock = self.inner.write().expect("Should not be poisoned");
		lock.transaction_version = value;
	}

	pub fn set_metadata(&self, value: Metadata) {
		let mut lock = self.inner.write().expect("Should not be poisoned");
		lock.metadata = value;
	}
}
