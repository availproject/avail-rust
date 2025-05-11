use crate::{
	avail, client::Client, config::AccountId, primitives::TransactionCall, SubmittableTransaction,
	SubmittableTransactionLike,
};

#[cfg(feature = "subxt_metadata")]
pub mod nom_pools;
#[cfg(feature = "subxt_metadata")]
pub mod session;
#[cfg(feature = "subxt_metadata")]
pub mod staking;
#[cfg(feature = "subxt_metadata")]
pub mod vector;

pub struct Transactions(pub(crate) Client);
impl Transactions {
	pub fn data_availability(&self) -> DataAvailability {
		DataAvailability(self.0.clone())
	}

	pub fn balances(&self) -> Balances {
		Balances(self.0.clone())
	}

	pub fn utility(&self) -> Utility {
		Utility(self.0.clone())
	}

	pub fn proxy(&self) -> Proxy {
		Proxy(self.0.clone())
	}
}

pub struct DataAvailability(Client);
impl DataAvailability {
	pub fn create_application_key(&self, key: Vec<u8>) -> SubmittableTransaction {
		avail::data_availability::tx::CreateApplicationKey { key }.to_submittable(self.0.clone())
	}

	pub fn submit_data(&self, data: Vec<u8>) -> SubmittableTransaction {
		avail::data_availability::tx::SubmitData { data }.to_submittable(self.0.clone())
	}
}

pub struct Balances(Client);
impl Balances {
	pub fn transfer_allow_death(&self, dest: AccountId, amount: u128) -> SubmittableTransaction {
		avail::balances::tx::TransferAllowDeath {
			dest: dest.into(),
			amount,
		}
		.to_submittable(self.0.clone())
	}

	pub fn transfer_keep_alive(&self, dest: AccountId, amount: u128) -> SubmittableTransaction {
		avail::balances::tx::TransferKeepAlive {
			dest: dest.into(),
			amount,
		}
		.to_submittable(self.0.clone())
	}

	pub fn transfer_all(&self, dest: AccountId, keep_alive: bool) -> SubmittableTransaction {
		avail::balances::tx::TransferAll {
			dest: dest.into(),
			keep_alive,
		}
		.to_submittable(self.0.clone())
	}
}

pub struct Utility(Client);
impl Utility {
	pub fn batch(&self, calls: Vec<TransactionCall>) -> SubmittableTransaction {
		avail::utility::tx::Batch { calls }.to_submittable(self.0.clone())
	}

	pub fn batch_call(&self, calls: Vec<TransactionCall>) -> SubmittableTransaction {
		avail::utility::tx::BatchAll { calls }.to_submittable(self.0.clone())
	}

	pub fn force_batch(&self, calls: Vec<TransactionCall>) -> SubmittableTransaction {
		avail::utility::tx::ForceBatch { calls }.to_submittable(self.0.clone())
	}
}

pub struct Proxy(Client);
impl Proxy {
	pub fn proxy(
		&self,
		id: AccountId,
		force_proxy_type: Option<avail::proxy::types::ProxyType>,
		call: TransactionCall,
	) -> SubmittableTransaction {
		avail::proxy::tx::Proxy {
			id: id.into(),
			force_proxy_type,
			call,
		}
		.to_submittable(self.0.clone())
	}

	pub fn add_proxy(
		&self,
		id: AccountId,
		proxy_type: avail::proxy::types::ProxyType,
		delay: u32,
	) -> SubmittableTransaction {
		avail::proxy::tx::AddProxy {
			id: id.into(),
			proxy_type,
			delay,
		}
		.to_submittable(self.0.clone())
	}

	pub fn remove_proxy(
		&self,
		delegate: AccountId,
		proxy_type: avail::proxy::types::ProxyType,
		delay: u32,
	) -> SubmittableTransaction {
		avail::proxy::tx::RemoveProxy {
			delegate: delegate.into(),
			proxy_type,
			delay,
		}
		.to_submittable(self.0.clone())
	}

	pub fn remove_proxies(&self) -> SubmittableTransaction {
		avail::proxy::tx::RemoveProxies {}.to_submittable(self.0.clone())
	}

	pub fn create_pure(
		&self,
		proxy_type: avail::proxy::types::ProxyType,
		delay: u32,
		index: u16,
	) -> SubmittableTransaction {
		avail::proxy::tx::CreatePure {
			proxy_type,
			delay,
			index,
		}
		.to_submittable(self.0.clone())
	}

	pub fn kill_pure(
		&self,
		spawner: AccountId,
		proxy_type: avail::proxy::types::ProxyType,
		index: u16,
		height: u32,
		ext_index: u32,
	) -> SubmittableTransaction {
		avail::proxy::tx::KillPure {
			spawner: spawner.into(),
			proxy_type,
			index,
			height,
			ext_index,
		}
		.to_submittable(self.0.clone())
	}
}
