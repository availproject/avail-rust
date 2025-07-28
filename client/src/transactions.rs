use crate::{Client, SubmittableTransaction, SubmittableTransactionLike};
use avail_rust_core::{AccountId, H256, TransactionCall, avail};

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

	pub fn vector(&self) -> Vector {
		Vector(self.0.clone())
	}

	pub fn system(&self) -> System {
		System(self.0.clone())
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
		avail::balances::tx::TransferAllowDeath { dest: dest.into(), value: amount.into() }
			.to_submittable(self.0.clone())
	}

	pub fn transfer_keep_alive(&self, dest: AccountId, amount: u128) -> SubmittableTransaction {
		avail::balances::tx::TransferKeepAlive { dest: dest.into(), value: amount.into() }
			.to_submittable(self.0.clone())
	}

	pub fn transfer_all(&self, dest: AccountId, keep_alive: bool) -> SubmittableTransaction {
		avail::balances::tx::TransferAll { dest: dest.into(), keep_alive }.to_submittable(self.0.clone())
	}
}

pub struct Utility(Client);
impl Utility {
	pub fn batch(&self, calls: Vec<TransactionCall>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::Batch::new();
		batch.add_calls(calls);
		batch.to_submittable(self.0.clone())
	}

	pub fn batch_all(&self, calls: Vec<TransactionCall>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::BatchAll::new();
		batch.add_calls(calls);
		batch.to_submittable(self.0.clone())
	}

	pub fn force_batch(&self, calls: Vec<TransactionCall>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::ForceBatch::new();
		batch.add_calls(calls);
		batch.to_submittable(self.0.clone())
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
		avail::proxy::tx::Proxy { id: id.into(), force_proxy_type, call }.to_submittable(self.0.clone())
	}

	pub fn add_proxy(
		&self,
		id: AccountId,
		proxy_type: avail::proxy::types::ProxyType,
		delay: u32,
	) -> SubmittableTransaction {
		avail::proxy::tx::AddProxy { id: id.into(), proxy_type, delay }.to_submittable(self.0.clone())
	}

	pub fn remove_proxy(
		&self,
		delegate: AccountId,
		proxy_type: avail::proxy::types::ProxyType,
		delay: u32,
	) -> SubmittableTransaction {
		avail::proxy::tx::RemoveProxy { delegate: delegate.into(), proxy_type, delay }.to_submittable(self.0.clone())
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
		avail::proxy::tx::CreatePure { proxy_type, delay, index }.to_submittable(self.0.clone())
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
			height: height.into(),
			ext_index: ext_index.into(),
		}
		.to_submittable(self.0.clone())
	}
}

pub struct Vector(Client);
impl Vector {
	pub fn batch(
		&self,
		function_id: H256,
		input: Vec<u8>,
		output: Vec<u8>,
		proof: Vec<u8>,
		slot: u64,
	) -> SubmittableTransaction {
		avail::vector::tx::FulfillCall { function_id, input, output, proof, slot: slot.into() }
			.to_submittable(self.0.clone())
	}

	pub fn execute(
		&self,
		slot: u64,
		addr_message: avail::vector::types::AddressedMessage,
		account_proof: Vec<Vec<u8>>,
		storage_proof: Vec<Vec<u8>>,
	) -> SubmittableTransaction {
		avail::vector::tx::Execute {
			slot: slot.into(),
			addr_message,
			account_proof,
			storage_proof,
		}
		.to_submittable(self.0.clone())
	}

	pub fn source_chain_froze(&self, source_chain_id: u32, frozen: bool) -> SubmittableTransaction {
		avail::vector::tx::SourceChainFroze { source_chain_id: source_chain_id.into(), frozen }
			.to_submittable(self.0.clone())
	}

	pub fn send_message(
		&self,
		slot: u64,
		message: avail::vector::types::Message,
		to: H256,
		domain: u32,
	) -> SubmittableTransaction {
		avail::vector::tx::SendMessage { slot: slot.into(), message, to, domain: domain.into() }
			.to_submittable(self.0.clone())
	}

	pub fn set_poseidon_hash(&self, period: u64, poseidon_hash: Vec<u8>) -> SubmittableTransaction {
		avail::vector::tx::SetPoseidonHash { period: period.into(), poseidon_hash }.to_submittable(self.0.clone())
	}

	pub fn set_broadcaster(&self, broadcaster_domain: u32, broadcaster: H256) -> SubmittableTransaction {
		avail::vector::tx::SetBroadcaster { broadcaster_domain: broadcaster_domain.into(), broadcaster }
			.to_submittable(self.0.clone())
	}

	pub fn set_whitelisted_domains(&self, value: Vec<u32>) -> SubmittableTransaction {
		avail::vector::tx::SetWhitelistedDomains { value }.to_submittable(self.0.clone())
	}

	pub fn set_configuration(&self, value: avail::vector::types::Configuration) -> SubmittableTransaction {
		avail::vector::tx::SetConfiguration { value }.to_submittable(self.0.clone())
	}

	pub fn set_function_ids(&self, value: Option<(H256, H256)>) -> SubmittableTransaction {
		avail::vector::tx::SetFunctionIds { value }.to_submittable(self.0.clone())
	}

	pub fn set_step_verification_key(&self, value: Option<Vec<u8>>) -> SubmittableTransaction {
		avail::vector::tx::SetStepVerificationKey { value }.to_submittable(self.0.clone())
	}

	pub fn set_updater(&self, updater: H256) -> SubmittableTransaction {
		avail::vector::tx::SetUpdater { updater }.to_submittable(self.0.clone())
	}

	pub fn fulfill(&self, proof: Vec<u8>, public_values: Vec<u8>) -> SubmittableTransaction {
		avail::vector::tx::Fulfill { proof, public_values }.to_submittable(self.0.clone())
	}

	pub fn set_sp1_verification_key(&self, sp1_vk: H256) -> SubmittableTransaction {
		avail::vector::tx::SetSp1VerificationKey { sp1_vk }.to_submittable(self.0.clone())
	}

	pub fn set_sync_committee_hash(&self, period: u64, hash: H256) -> SubmittableTransaction {
		avail::vector::tx::SetSyncCommitteeHash { period, hash }.to_submittable(self.0.clone())
	}

	pub fn enable_mock(&self, value: bool) -> SubmittableTransaction {
		avail::vector::tx::EnableMock { value }.to_submittable(self.0.clone())
	}

	pub fn mock_fulfill(&self, public_values: Vec<u8>) -> SubmittableTransaction {
		avail::vector::tx::MockFulfill { public_values }.to_submittable(self.0.clone())
	}
}

pub struct System(Client);
impl System {
	pub fn remark(&self, remark: Vec<u8>) -> SubmittableTransaction {
		avail::system::tx::Remark { remark }.to_submittable(self.0.clone())
	}

	pub fn set_code(&self, code: Vec<u8>) -> SubmittableTransaction {
		avail::system::tx::SetCode { code }.to_submittable(self.0.clone())
	}

	pub fn set_code_without_checks(&self, code: Vec<u8>) -> SubmittableTransaction {
		avail::system::tx::SetCodeWithoutChecks { code }.to_submittable(self.0.clone())
	}

	pub fn remark_with_event(&self, remark: Vec<u8>) -> SubmittableTransaction {
		avail::system::tx::RemarkWithEvent { remark }.to_submittable(self.0.clone())
	}
}
