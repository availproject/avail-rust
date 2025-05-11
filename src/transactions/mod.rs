use crate::{
	api_dev_custom::TransactionCallLike, avail, client::Client, config::AccountId, primitives::TransactionCall,
	SubmittableTransaction,
};

#[cfg(feature = "subxt_metadata")]
pub mod balances;
#[cfg(feature = "subxt_metadata")]
pub mod nom_pools;
#[cfg(feature = "subxt_metadata")]
pub mod proxy;
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
}

pub struct DataAvailability(Client);
impl DataAvailability {
	pub fn create_application_key(&self, key: Vec<u8>) -> SubmittableTransaction {
		let call = avail::data_availability::tx::CreateApplicationKey { key }.to_call();
		SubmittableTransaction::new(self.0.clone(), call)
	}

	pub fn submit_data(&self, data: Vec<u8>) -> SubmittableTransaction {
		let call = avail::data_availability::tx::SubmitData { data }.to_call();
		SubmittableTransaction::new(self.0.clone(), call)
	}
}

pub struct Balances(Client);
impl Balances {
	pub fn transfer_allow_death(&self, dest: AccountId, amount: u128) -> SubmittableTransaction {
		let call = avail::balances::tx::TransferAllowDeath {
			dest: dest.into(),
			amount,
		}
		.to_call();
		SubmittableTransaction::new(self.0.clone(), call)
	}

	pub fn transfer_keep_alive(&self, dest: AccountId, amount: u128) -> SubmittableTransaction {
		let call = avail::balances::tx::TransferKeepAlive {
			dest: dest.into(),
			amount,
		}
		.to_call();
		SubmittableTransaction::new(self.0.clone(), call)
	}

	pub fn transfer_all(&self, dest: AccountId, keep_alive: bool) -> SubmittableTransaction {
		let call = avail::balances::tx::TransferAll {
			dest: dest.into(),
			keep_alive,
		}
		.to_call();
		SubmittableTransaction::new(self.0.clone(), call)
	}
}

pub struct Utility(Client);
impl Utility {
	pub fn batch(&self, calls: Vec<TransactionCall>) -> SubmittableTransaction {
		let call = avail::utility::tx::Batch { calls }.to_call();
		SubmittableTransaction::new(self.0.clone(), call)
	}

	pub fn batch_call(&self, calls: Vec<TransactionCall>) -> SubmittableTransaction {
		let call = avail::utility::tx::BatchAll { calls }.to_call();
		SubmittableTransaction::new(self.0.clone(), call)
	}

	pub fn force_batch(&self, calls: Vec<TransactionCall>) -> SubmittableTransaction {
		let call = avail::utility::tx::ForceBatch { calls }.to_call();
		SubmittableTransaction::new(self.0.clone(), call)
	}
}
