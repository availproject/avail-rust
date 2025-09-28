use crate::{Client, SubmittableTransaction};
use avail_rust_core::{
	AccountId, AccountIdLike, ExtrinsicCall, H256, MultiAddress,
	avail::{
		self,
		multisig::types::Timepoint,
		nomination_pools::types::{BondExtraValue, ClaimPermission, ConfigOpAccount, PoolState},
		proxy::types::ProxyType,
		staking::types::{RewardDestination, ValidatorPrefs},
	},
	types::{
		HashString,
		metadata::{MultiAddressLike, StringOrBytes},
		substrate::Weight,
	},
};

pub struct TransactionApi(pub(crate) Client);
impl TransactionApi {
	pub fn balances(&self) -> Balances {
		Balances(self.0.clone())
	}

	pub fn data_availability(&self) -> DataAvailability {
		DataAvailability(self.0.clone())
	}

	pub fn multisig(&self) -> Multisig {
		Multisig(self.0.clone())
	}

	pub fn utility(&self) -> Utility {
		Utility(self.0.clone())
	}

	pub fn proxy(&self) -> Proxy {
		Proxy(self.0.clone())
	}

	pub fn staking(&self) -> Staking {
		Staking(self.0.clone())
	}

	pub fn vector(&self) -> Vector {
		Vector(self.0.clone())
	}

	pub fn system(&self) -> System {
		System(self.0.clone())
	}

	pub fn nomination_pools(&self) -> NominationPools {
		NominationPools(self.0.clone())
	}

	pub fn session(&self) -> Session {
		Session(self.0.clone())
	}
}

pub struct Session(Client);
impl Session {
	pub fn set_key(
		&self,
		babe: impl Into<HashString>,
		grandpa: impl Into<HashString>,
		authority_discovery: impl Into<HashString>,
		im_online: impl Into<HashString>,
		proof: Vec<u8>,
	) -> SubmittableTransaction {
		let babe: HashString = babe.into();
		let babe: H256 = babe.try_into().expect("Invalid string for H256");

		let grandpa: HashString = grandpa.into();
		let grandpa: H256 = grandpa.try_into().expect("Invalid string for H256");

		let authority_discovery: HashString = authority_discovery.into();
		let authority_discovery: H256 = authority_discovery.try_into().expect("Invalid string for H256");

		let im_online: HashString = im_online.into();
		let im_online: H256 = im_online.try_into().expect("Invalid string for H256");

		let value = avail::session::tx::SetKeys { babe, grandpa, authority_discovery, im_online, proof };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn purge_key(&self) -> SubmittableTransaction {
		let value = avail::session::tx::PurgeKeys {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

pub struct NominationPools(Client);
impl NominationPools {
	pub fn bond_extra(&self, value: BondExtraValue) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::BondExtra { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn bond_extra_other(
		&self,
		member: impl Into<MultiAddressLike>,
		value: BondExtraValue,
	) -> SubmittableTransaction {
		let member: MultiAddressLike = member.into();
		let member: MultiAddress = member.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::nomination_pools::tx::BondExtraOther { member, value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn chill(&self, pool_id: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::Chill { pool_id };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn claim_commission(&self, pool_id: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::ClaimCommission { pool_id };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn claim_payout(&self) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::ClaimPayout {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn claim_payout_other(&self, owner: impl Into<AccountIdLike>) -> SubmittableTransaction {
		let owner: AccountIdLike = owner.into();
		let owner: AccountId = owner.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::nomination_pools::tx::ClaimPayoutOther { owner };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn create(
		&self,
		amount: u128,
		root: impl Into<MultiAddressLike>,
		nominator: impl Into<MultiAddressLike>,
		bouncer: impl Into<MultiAddressLike>,
	) -> SubmittableTransaction {
		let root: MultiAddressLike = root.into();
		let root: MultiAddress = root.try_into().expect("Malformed string is passed for AccountId");
		let nominator: MultiAddressLike = nominator.into();
		let nominator: MultiAddress = nominator.try_into().expect("Malformed string is passed for AccountId");
		let bouncer: MultiAddressLike = bouncer.into();
		let bouncer: MultiAddress = bouncer.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::nomination_pools::tx::Create { amount, root, nominator, bouncer };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn create_with_pool_id(
		&self,
		amount: u128,
		root: impl Into<MultiAddressLike>,
		nominator: impl Into<MultiAddressLike>,
		bouncer: impl Into<MultiAddressLike>,
		pool_id: u32,
	) -> SubmittableTransaction {
		let root: MultiAddressLike = root.into();
		let root: MultiAddress = root.try_into().expect("Malformed string is passed for AccountId");
		let nominator: MultiAddressLike = nominator.into();
		let nominator: MultiAddress = nominator.try_into().expect("Malformed string is passed for AccountId");
		let bouncer: MultiAddressLike = bouncer.into();
		let bouncer: MultiAddress = bouncer.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::nomination_pools::tx::CreateWithPoolId { amount, root, nominator, bouncer, pool_id };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn join(&self, amount: u128, pool_id: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::Join { amount, pool_id };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn nominate(&self, pool_id: u32, validators: Vec<impl Into<AccountIdLike>>) -> SubmittableTransaction {
		let validators: Vec<AccountIdLike> = validators.into_iter().map(|x| x.into()).collect();
		let validators: Result<Vec<AccountId>, _> = validators.into_iter().map(AccountId::try_from).collect();
		let validators = validators.expect("Malformed string is passed for AccountId");

		let value = avail::nomination_pools::tx::Nominate { pool_id, validators };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_claim_permission(&self, permission: ClaimPermission) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetClaimPermission { permission };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_commission(&self, pool_id: u32, new_commission: Option<(u32, AccountIdLike)>) -> SubmittableTransaction {
		let new_commission =
			new_commission.map(|x| (x.0, AccountId::try_from(x.1).expect("Malformed string is passed for AccountId")));
		let value = avail::nomination_pools::tx::SetCommission { pool_id, new_commission };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_commission_change_rate(
		&self,
		pool_id: u32,
		max_increase: u32,
		min_delay: u32,
	) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetCommissionChangeRate { pool_id, max_increase, min_delay };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_commission_max(&self, pool_id: u32, max_commission: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetCommissionMax { pool_id, max_commission };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_metadata<'a>(&self, pool_id: u32, metadata: impl Into<StringOrBytes<'a>>) -> SubmittableTransaction {
		let metadata: StringOrBytes = metadata.into();
		let metadata: Vec<u8> = metadata.into();
		let value = avail::nomination_pools::tx::SetMetadata { pool_id, metadata };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_state(&self, pool_id: u32, state: PoolState) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetState { pool_id, state };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn unbond(
		&self,
		member_account: impl Into<MultiAddressLike>,
		unbonding_points: u128,
	) -> SubmittableTransaction {
		let member_account: MultiAddressLike = member_account.into();
		let member_account: MultiAddress = member_account
			.try_into()
			.expect("Malformed string is passed for AccountId");

		let value = avail::nomination_pools::tx::Unbond { member_account, unbonding_points };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn update_roles(
		&self,
		pool_id: u32,
		new_root: ConfigOpAccount,
		new_nominator: ConfigOpAccount,
		new_bouncer: ConfigOpAccount,
	) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::UpdateRoles { pool_id, new_root, new_nominator, new_bouncer };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn withdraw_unbonded(
		&self,
		member_account: impl Into<MultiAddressLike>,
		num_slashing_spans: u32,
	) -> SubmittableTransaction {
		let member_account: MultiAddressLike = member_account.into();
		let member_account: MultiAddress = member_account
			.try_into()
			.expect("Malformed string is passed for AccountId");

		let value = avail::nomination_pools::tx::WithdrawUnbonded { member_account, num_slashing_spans };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

pub struct Staking(Client);
impl Staking {
	pub fn bond(&self, value: u128, payee: RewardDestination) -> SubmittableTransaction {
		let value = avail::staking::tx::Bond { value, payee };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn bond_extra(&self, value: u128) -> SubmittableTransaction {
		let value = avail::staking::tx::BondExtra { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn unbond(&self, value: u128) -> SubmittableTransaction {
		let value = avail::staking::tx::Unbond { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn rebond(&self, value: u128) -> SubmittableTransaction {
		let value = avail::staking::tx::Rebond { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn validate(&self, commission: u32, blocked: bool) -> SubmittableTransaction {
		let value = avail::staking::tx::Validate { prefs: ValidatorPrefs { commission, blocked } };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn nominate(&self, targets: Vec<impl Into<MultiAddressLike>>) -> SubmittableTransaction {
		let targets: Vec<MultiAddressLike> = targets.into_iter().map(|x| x.into()).collect();
		let targets: Result<Vec<MultiAddress>, _> = targets.into_iter().map(MultiAddress::try_from).collect();
		let targets = targets.expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::Nominate { targets };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn payout_stakers(&self, validator_stash: impl Into<AccountIdLike>, era: u32) -> SubmittableTransaction {
		let validator_stash: AccountIdLike = validator_stash.into();
		let validator_stash = AccountId::try_from(validator_stash).expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::PayoutStakers { validator_stash, era };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_controller(&self) -> SubmittableTransaction {
		let value = avail::staking::tx::SetController {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn set_payee(&self, payee: RewardDestination) -> SubmittableTransaction {
		let value = avail::staking::tx::SetPayee { payee };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn chill(&self) -> SubmittableTransaction {
		let value = avail::staking::tx::Chill {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn chill_other(&self, stash: impl Into<AccountIdLike>) -> SubmittableTransaction {
		let stash: AccountIdLike = stash.into();
		let stash = AccountId::try_from(stash).expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::ChillOther { stash };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn withdraw_unbonded(&self, num_slashing_spans: u32) -> SubmittableTransaction {
		let value = avail::staking::tx::WithdrawUnbonded { num_slashing_spans };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn reap_stash(&self, stash: impl Into<AccountIdLike>, num_slashing_spans: u32) -> SubmittableTransaction {
		let stash: AccountIdLike = stash.into();
		let stash = AccountId::try_from(stash).expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::ReapStash { stash, num_slashing_spans };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn kick(&self, who: Vec<impl Into<MultiAddressLike>>) -> SubmittableTransaction {
		let who: Vec<MultiAddressLike> = who.into_iter().map(|x| x.into()).collect();
		let who: Result<Vec<MultiAddress>, _> = who.into_iter().map(MultiAddress::try_from).collect();
		let who = who.expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::Kick { who };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn force_apply_min_commission(&self, validator_stash: impl Into<AccountIdLike>) -> SubmittableTransaction {
		let validator_stash: AccountIdLike = validator_stash.into();
		let validator_stash = AccountId::try_from(validator_stash).expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::ForceApplyMinCommission { validator_stash };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn payout_stakers_by_page(
		&self,
		validator_stash: impl Into<AccountIdLike>,
		era: u32,
		page: u32,
	) -> SubmittableTransaction {
		let validator_stash: AccountIdLike = validator_stash.into();
		let validator_stash = AccountId::try_from(validator_stash).expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::PayoutStakersByPage { validator_stash, era, page };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

pub struct Balances(Client);
impl Balances {
	pub fn transfer_allow_death(&self, dest: impl Into<MultiAddressLike>, amount: u128) -> SubmittableTransaction {
		let dest: MultiAddressLike = dest.into();
		let dest: MultiAddress = dest.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::balances::tx::TransferAllowDeath { dest, value: amount };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn transfer_keep_alive(&self, dest: impl Into<MultiAddressLike>, amount: u128) -> SubmittableTransaction {
		let dest: MultiAddressLike = dest.into();
		let dest: MultiAddress = dest.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::balances::tx::TransferKeepAlive { dest, value: amount };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn transfer_all(&self, dest: impl Into<MultiAddressLike>, keep_alive: bool) -> SubmittableTransaction {
		let dest: MultiAddressLike = dest.into();
		let dest: MultiAddress = dest.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::balances::tx::TransferAll { dest, keep_alive };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

pub struct Multisig(Client);
impl Multisig {
	pub fn approve_as_multi(
		&self,
		threshold: u16,
		other_signatories: Vec<impl Into<AccountIdLike>>,
		maybe_timepoint: Option<Timepoint>,
		call_hash: impl Into<HashString>,
		max_weight: Weight,
	) -> SubmittableTransaction {
		fn inner(
			client: Client,
			threshold: u16,
			other_signatories: Vec<AccountIdLike>,
			maybe_timepoint: Option<Timepoint>,
			call_hash: HashString,
			max_weight: Weight,
		) -> SubmittableTransaction {
			let other_signatories: Result<Vec<AccountId>, _> =
				other_signatories.into_iter().map(|x| x.try_into()).collect();
			let mut other_signatories = other_signatories.expect("Malformed string is passed for AccountId");
			other_signatories.sort_by(|x, y| x.cmp(&y));

			let call_hash: H256 = call_hash.try_into().expect("Malformed string is passed for H256");

			let value = avail::multisig::tx::ApproveAsMulti {
				threshold,
				other_signatories,
				maybe_timepoint,
				call_hash,
				max_weight,
			};
			SubmittableTransaction::from_encodable(client, value)
		}

		let other_signatories: Vec<AccountIdLike> = other_signatories.into_iter().map(|x| x.into()).collect();
		let call_hash: HashString = call_hash.into();
		inner(self.0.clone(), threshold, other_signatories, maybe_timepoint, call_hash, max_weight)
	}

	pub fn as_multi(
		&self,
		threshold: u16,
		other_signatories: Vec<impl Into<AccountIdLike>>,
		maybe_timepoint: Option<Timepoint>,
		call: impl Into<ExtrinsicCall>,
		max_weight: Weight,
	) -> SubmittableTransaction {
		fn inner(
			client: Client,
			threshold: u16,
			other_signatories: Vec<AccountIdLike>,
			maybe_timepoint: Option<Timepoint>,
			call: ExtrinsicCall,
			max_weight: Weight,
		) -> SubmittableTransaction {
			let other_signatories: Result<Vec<AccountId>, _> =
				other_signatories.into_iter().map(|x| x.try_into()).collect();
			let mut other_signatories = other_signatories.expect("Malformed string is passed for AccountId");
			other_signatories.sort_by(|x, y| x.cmp(&y));

			let value = avail::multisig::tx::AsMulti {
				threshold,
				other_signatories,
				maybe_timepoint,
				call,
				max_weight,
			};
			SubmittableTransaction::from_encodable(client, value)
		}

		let other_signatories: Vec<AccountIdLike> = other_signatories.into_iter().map(|x| x.into()).collect();
		inner(self.0.clone(), threshold, other_signatories, maybe_timepoint, call.into(), max_weight)
	}

	pub fn as_multi_threshold_1(
		&self,
		other_signatories: Vec<impl Into<AccountIdLike>>,
		call: impl Into<ExtrinsicCall>,
	) -> SubmittableTransaction {
		fn inner(client: Client, other_signatories: Vec<AccountIdLike>, call: ExtrinsicCall) -> SubmittableTransaction {
			let other_signatories: Result<Vec<AccountId>, _> =
				other_signatories.into_iter().map(|x| x.try_into()).collect();
			let mut other_signatories = other_signatories.expect("Malformed string is passed for AccountId");
			other_signatories.sort_by(|x, y| x.cmp(&y));

			let value = avail::multisig::tx::AsMultiThreshold1 { other_signatories, call };
			SubmittableTransaction::from_encodable(client, value)
		}

		let other_signatories: Vec<AccountIdLike> = other_signatories.into_iter().map(|x| x.into()).collect();
		inner(self.0.clone(), other_signatories, call.into())
	}

	pub fn cancel_as_multi(
		&self,
		threshold: u16,
		other_signatories: Vec<impl Into<AccountIdLike>>,
		timepoint: Timepoint,
		call_hash: impl Into<HashString>,
	) -> SubmittableTransaction {
		fn inner(
			client: Client,
			threshold: u16,
			other_signatories: Vec<AccountIdLike>,
			timepoint: Timepoint,
			call_hash: HashString,
		) -> SubmittableTransaction {
			let other_signatories: Result<Vec<AccountId>, _> =
				other_signatories.into_iter().map(|x| x.try_into()).collect();
			let mut other_signatories = other_signatories.expect("Malformed string is passed for AccountId");
			other_signatories.sort_by(|x, y| x.cmp(&y));

			let call_hash: H256 = call_hash.try_into().expect("Malformed string is passed for H256");

			let value = avail::multisig::tx::CancelAsMulti { threshold, other_signatories, timepoint, call_hash };
			SubmittableTransaction::from_encodable(client, value)
		}

		let other_signatories: Vec<AccountIdLike> = other_signatories.into_iter().map(|x| x.into()).collect();
		let call_hash: HashString = call_hash.into();
		inner(self.0.clone(), threshold, other_signatories, timepoint, call_hash)
	}
}

pub struct DataAvailability(Client);
impl DataAvailability {
	pub fn create_application_key<'a>(&self, key: impl Into<StringOrBytes<'a>>) -> SubmittableTransaction {
		let key: Vec<u8> = Into::<StringOrBytes>::into(key).into();
		let value = avail::data_availability::tx::CreateApplicationKey { key };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn submit_data<'a>(&self, data: impl Into<StringOrBytes<'a>>) -> SubmittableTransaction {
		let data: Vec<u8> = Into::<StringOrBytes>::into(data).into();
		let value = avail::data_availability::tx::SubmitData { data };
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
		id: impl Into<MultiAddressLike>,
		force_proxy_type: Option<ProxyType>,
		call: impl Into<ExtrinsicCall>,
	) -> SubmittableTransaction {
		let id: MultiAddressLike = id.into();
		let id: MultiAddress = id.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::proxy::tx::Proxy { id, force_proxy_type, call: call.into() };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn add_proxy(
		&self,
		id: impl Into<MultiAddressLike>,
		proxy_type: ProxyType,
		delay: u32,
	) -> SubmittableTransaction {
		let id: MultiAddressLike = id.into();
		let id: MultiAddress = id.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::proxy::tx::AddProxy { id, proxy_type, delay };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn remove_proxy(
		&self,
		delegate: impl Into<MultiAddressLike>,
		proxy_type: ProxyType,
		delay: u32,
	) -> SubmittableTransaction {
		let delegate: MultiAddressLike = delegate.into();
		let delegate: MultiAddress = delegate.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::proxy::tx::RemoveProxy { delegate, proxy_type, delay };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn remove_proxies(&self) -> SubmittableTransaction {
		let value = avail::proxy::tx::RemoveProxies {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn create_pure(&self, proxy_type: ProxyType, delay: u32, index: u16) -> SubmittableTransaction {
		let value = avail::proxy::tx::CreatePure { proxy_type, delay, index };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn kill_pure(
		&self,
		spawner: impl Into<MultiAddressLike>,
		proxy_type: ProxyType,
		index: u16,
		height: u32,
		ext_index: u32,
	) -> SubmittableTransaction {
		let spawner: MultiAddressLike = spawner.into();
		let spawner: MultiAddress = spawner.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::proxy::tx::KillPure { spawner, proxy_type, index, height, ext_index };
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
		let value = avail::vector::tx::FulfillCall { function_id, input, output, proof, slot };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn execute(
		&self,
		slot: u64,
		addr_message: avail::vector::types::AddressedMessage,
		account_proof: Vec<Vec<u8>>,
		storage_proof: Vec<Vec<u8>>,
	) -> SubmittableTransaction {
		let value = avail::vector::tx::Execute { slot, addr_message, account_proof, storage_proof };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn source_chain_froze(&self, source_chain_id: u32, frozen: bool) -> SubmittableTransaction {
		let value = avail::vector::tx::SourceChainFroze { source_chain_id, frozen };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn send_message(
		&self,
		message: avail::vector::types::Message,
		to: impl Into<HashString>,
		domain: u32,
	) -> SubmittableTransaction {
		let to: HashString = to.into();
		let to: H256 = to.try_into().expect("Malformed string is passed for H256");

		let value = avail::vector::tx::SendMessage { message, to, domain };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	pub fn failed_send_message_txs(&self, failed_txs: Vec<u32>) -> SubmittableTransaction {
		let value = avail::vector::tx::FailedSendMessageTxs { failed_txs };
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
