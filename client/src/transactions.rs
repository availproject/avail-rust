use crate::{Client, SubmittableTransaction};
use avail_rust_core::{AccountId, ExtrinsicCall, H256, avail};

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
		let value = avail::data_availability::tx::CreateApplicationKey { key };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn submit_data(&self, data: Vec<u8>) -> SubmittableTransaction {
		let value = avail::data_availability::tx::SubmitData { data };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

pub struct Balances(Client);
impl Balances {
	pub fn transfer_allow_death(&self, dest: AccountId, amount: u128) -> SubmittableTransaction {
		let value = avail::balances::tx::TransferAllowDeath { dest: dest.into(), value: amount };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn transfer_keep_alive(&self, dest: AccountId, amount: u128) -> SubmittableTransaction {
		let value = avail::balances::tx::TransferKeepAlive { dest: dest.into(), value: amount };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn transfer_all(&self, dest: AccountId, keep_alive: bool) -> SubmittableTransaction {
		let value = avail::balances::tx::TransferAll { dest: dest.into(), keep_alive };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

pub struct Utility(Client);
impl Utility {
	pub fn batch(&self, calls: Vec<impl Into<ExtrinsicCall>>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::Batch::new();
		batch.add_calls(calls.into_iter().map(|x| x.into()).collect());
		SubmittableTransaction::from_encodable(self.0.clone(), batch)
	}

	pub fn batch_all(&self, calls: Vec<impl Into<ExtrinsicCall>>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::BatchAll::new();
		batch.add_calls(calls.into_iter().map(|x| x.into()).collect());
		SubmittableTransaction::from_encodable(self.0.clone(), batch)
	}

	pub fn force_batch(&self, calls: Vec<impl Into<ExtrinsicCall>>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::ForceBatch::new();
		batch.add_calls(calls.into_iter().map(|x| x.into()).collect());
		SubmittableTransaction::from_encodable(self.0.clone(), batch)
	}
}

pub struct Proxy(Client);
impl Proxy {
	pub fn proxy(
		&self,
		id: AccountId,
		force_proxy_type: Option<avail::proxy::types::ProxyType>,
		call: ExtrinsicCall,
	) -> SubmittableTransaction {
		let value = avail::proxy::tx::Proxy { id: id.into(), force_proxy_type, call };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn add_proxy(
		&self,
		id: AccountId,
		proxy_type: avail::proxy::types::ProxyType,
		delay: u32,
	) -> SubmittableTransaction {
		let value = avail::proxy::tx::AddProxy { id: id.into(), proxy_type, delay };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn remove_proxy(
		&self,
		delegate: AccountId,
		proxy_type: avail::proxy::types::ProxyType,
		delay: u32,
	) -> SubmittableTransaction {
		let value = avail::proxy::tx::RemoveProxy { delegate: delegate.into(), proxy_type, delay };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn remove_proxies(&self) -> SubmittableTransaction {
		let value = avail::proxy::tx::RemoveProxies {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn create_pure(
		&self,
		proxy_type: avail::proxy::types::ProxyType,
		delay: u32,
		index: u16,
	) -> SubmittableTransaction {
		let value = avail::proxy::tx::CreatePure { proxy_type, delay, index };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn kill_pure(
		&self,
		spawner: AccountId,
		proxy_type: avail::proxy::types::ProxyType,
		index: u16,
		height: u32,
		ext_index: u32,
	) -> SubmittableTransaction {
		let value = avail::proxy::tx::KillPure {
			spawner: spawner.into(),
			proxy_type,
			index,
			height: height.into(),
			ext_index: ext_index.into(),
		};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
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
		let value = avail::vector::tx::FulfillCall { function_id, input, output, proof, slot: slot.into() };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn execute(
		&self,
		slot: u64,
		addr_message: avail::vector::types::AddressedMessage,
		account_proof: Vec<Vec<u8>>,
		storage_proof: Vec<Vec<u8>>,
	) -> SubmittableTransaction {
		let value = avail::vector::tx::Execute {
			slot: slot.into(),
			addr_message,
			account_proof,
			storage_proof,
		};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn source_chain_froze(&self, source_chain_id: u32, frozen: bool) -> SubmittableTransaction {
		let value = avail::vector::tx::SourceChainFroze { source_chain_id: source_chain_id.into(), frozen };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn send_message(
		&self,
		slot: u64,
		message: avail::vector::types::Message,
		to: H256,
		domain: u32,
	) -> SubmittableTransaction {
		let value = avail::vector::tx::SendMessage { slot: slot.into(), message, to, domain: domain.into() };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_poseidon_hash(&self, period: u64, poseidon_hash: Vec<u8>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetPoseidonHash { period: period.into(), poseidon_hash };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_broadcaster(&self, broadcaster_domain: u32, broadcaster: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetBroadcaster { broadcaster_domain: broadcaster_domain.into(), broadcaster };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_whitelisted_domains(&self, value: Vec<u32>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetWhitelistedDomains { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_configuration(&self, value: avail::vector::types::Configuration) -> SubmittableTransaction {
		let value = avail::vector::tx::SetConfiguration { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_function_ids(&self, value: Option<(H256, H256)>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetFunctionIds { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_step_verification_key(&self, value: Option<Vec<u8>>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetStepVerificationKey { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_updater(&self, updater: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetUpdater { updater };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn fulfill(&self, proof: Vec<u8>, public_values: Vec<u8>) -> SubmittableTransaction {
		let value = avail::vector::tx::Fulfill { proof, public_values };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_sp1_verification_key(&self, sp1_vk: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetSp1VerificationKey { sp1_vk };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_sync_committee_hash(&self, period: u64, hash: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetSyncCommitteeHash { period, hash };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn enable_mock(&self, value: bool) -> SubmittableTransaction {
		let value = avail::vector::tx::EnableMock { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn mock_fulfill(&self, public_values: Vec<u8>) -> SubmittableTransaction {
		let value = avail::vector::tx::MockFulfill { public_values };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

pub struct System(Client);
impl System {
	pub fn remark(&self, remark: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::Remark { remark };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_code(&self, code: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::SetCode { code };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_code_without_checks(&self, code: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::SetCodeWithoutChecks { code };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn remark_with_event(&self, remark: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::RemarkWithEvent { remark };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

/* pub enum ExtrinsicCallLike {
	Call(ExtrinsicCall),
}

impl From<SubmittableTransaction> for ExtrinsicCallLike {
	fn from(value: SubmittableTransaction) -> Self {
		Self::from(value.call)
	}
}

impl From<&SubmittableTransaction> for ExtrinsicCallLike {
	fn from(value: &SubmittableTransaction) -> Self {
		Self::from(&value.call)
	}
}

impl From<ExtrinsicCall> for ExtrinsicCallLike {
	fn from(value: ExtrinsicCall) -> Self {
		Self::Call(value)
	}
}

impl From<&ExtrinsicCall> for ExtrinsicCallLike {
	fn from(value: &ExtrinsicCall) -> Self {
		Self::Call(value.clone())
	}
}

impl<T: HasHeader + Encode> From<T> for ExtrinsicCallLike {
	fn from(value: T) -> Self {
		let call = ExtrinsicCall::new(T::HEADER_INDEX.0, T::HEADER_INDEX.1, value.encode());
		Self::from(call)
	}
}
 */
