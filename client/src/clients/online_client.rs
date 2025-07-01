use crate::{subxt_core::Metadata, subxt_rpcs::RpcClient};
use avail_rust_core::{H256, ext::codec::Decode, rpc};
use std::sync::{Arc, RwLock};

#[cfg(feature = "subxt")]
use crate::config::AvailConfig;
#[cfg(feature = "subxt")]
use crate::subxt::OnlineClient as SubxtOnlineClient;

pub trait OnlineClientT: Send + Sync + 'static {
	fn genesis_hash(&self) -> H256;
	fn spec_version(&self) -> u32;
	fn transaction_version(&self) -> u32;
	fn metadata(&self) -> Metadata;

	fn set_genesis_hash(&self, value: H256);
	fn set_spec_version(&self, value: u32);
	fn set_transaction_version(&self, value: u32);
	fn set_metadata(&self, value: Metadata);
}

#[derive(Clone)]
pub struct OnlineClient {
	client: Arc<dyn OnlineClientT>,
}

impl std::ops::Deref for OnlineClient {
	type Target = dyn OnlineClientT;
	fn deref(&self) -> &Self::Target {
		&*self.client
	}
}

impl OnlineClientT for OnlineClient {
	fn genesis_hash(&self) -> H256 {
		self.client.genesis_hash()
	}

	fn spec_version(&self) -> u32 {
		self.client.spec_version()
	}

	fn transaction_version(&self) -> u32 {
		self.client.transaction_version()
	}

	fn metadata(&self) -> Metadata {
		self.client.metadata()
	}

	fn set_genesis_hash(&self, value: H256) {
		self.client.set_genesis_hash(value);
	}

	fn set_spec_version(&self, value: u32) {
		self.client.set_spec_version(value);
	}

	fn set_transaction_version(&self, value: u32) {
		self.client.set_transaction_version(value);
	}

	fn set_metadata(&self, value: Metadata) {
		self.client.set_metadata(value);
	}
}

impl From<SimpleOnlineClient> for OnlineClient {
	fn from(value: SimpleOnlineClient) -> Self {
		Self {
			client: Arc::new(value),
		}
	}
}

#[cfg(feature = "subxt")]
impl From<SubxtOnlineClient<AvailConfig>> for OnlineClient {
	fn from(value: SubxtOnlineClient<AvailConfig>) -> Self {
		Self {
			client: Arc::new(value),
		}
	}
}

#[derive(Clone)]
pub struct SimpleOnlineClient {
	pub inner: Arc<RwLock<SimpleOnlineClientInner>>,
}

#[derive(Clone)]
pub struct SimpleOnlineClientInner {
	pub genesis_hash: H256,
	pub spec_version: u32,
	pub transaction_version: u32,
	pub metadata: Metadata,
}

impl SimpleOnlineClient {
	pub async fn new(rpc_client: &RpcClient) -> Result<Self, avail_rust_core::Error> {
		let finalized_hash = rpc::chain::get_finalized_head(rpc_client).await?;
		let rpc_metadata = rpc::state::get_metadata(rpc_client, Some(finalized_hash)).await?;
		let genesis_hash = rpc::chainspec::v1_genesishash(rpc_client).await?;
		let runtime_version = rpc::state::get_runtime_version(rpc_client, Some(finalized_hash)).await?;

		let frame_metadata =
			frame_metadata::RuntimeMetadataPrefixed::decode(&mut rpc_metadata.as_slice()).map_err(|e| e.to_string())?;
		let metadata = Metadata::try_from(frame_metadata).map_err(|e| e.to_string())?;
		let inner = SimpleOnlineClientInner {
			genesis_hash,
			spec_version: runtime_version.spec_version,
			transaction_version: runtime_version.transaction_version,
			metadata,
		};
		Ok(Self {
			inner: Arc::new(RwLock::new(inner)),
		})
	}
}

impl OnlineClientT for SimpleOnlineClient {
	fn genesis_hash(&self) -> H256 {
		let lock = self.inner.read().expect("Should not be poisoned");
		lock.genesis_hash
	}

	fn spec_version(&self) -> u32 {
		let lock = self.inner.read().expect("Should not be poisoned");
		lock.spec_version
	}

	fn transaction_version(&self) -> u32 {
		let lock = self.inner.read().expect("Should not be poisoned");
		lock.transaction_version
	}

	fn metadata(&self) -> Metadata {
		let lock = self.inner.read().expect("Should not be poisoned");
		lock.metadata.clone()
	}

	fn set_genesis_hash(&self, value: H256) {
		let mut lock = self.inner.write().expect("Should not be poisoned");
		lock.genesis_hash = value;
	}

	fn set_spec_version(&self, value: u32) {
		let mut lock = self.inner.write().expect("Should not be poisoned");
		lock.spec_version = value;
	}

	fn set_transaction_version(&self, value: u32) {
		let mut lock = self.inner.write().expect("Should not be poisoned");
		lock.transaction_version = value;
	}

	fn set_metadata(&self, value: Metadata) {
		let mut lock = self.inner.write().expect("Should not be poisoned");
		lock.metadata = value;
	}
}

#[cfg(feature = "subxt")]
impl OnlineClientT for SubxtOnlineClient<AvailConfig> {
	fn genesis_hash(&self) -> H256 {
		self.genesis_hash()
	}

	fn spec_version(&self) -> u32 {
		self.runtime_version().spec_version
	}

	fn transaction_version(&self) -> u32 {
		self.runtime_version().transaction_version
	}

	fn metadata(&self) -> Metadata {
		self.metadata()
	}

	fn set_genesis_hash(&self, value: H256) {
		self.set_genesis_hash(value);
	}

	fn set_spec_version(&self, value: u32) {
		let mut runtime = self.runtime_version();
		runtime.spec_version = value;
		self.set_runtime_version(runtime);
	}

	fn set_transaction_version(&self, value: u32) {
		let mut runtime = self.runtime_version();
		runtime.transaction_version = value;
		self.set_runtime_version(runtime);
	}

	fn set_metadata(&self, value: Metadata) {
		self.set_metadata(value);
	}
}
