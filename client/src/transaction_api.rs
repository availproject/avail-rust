//! Builders for transactions targeting specific Avail pallets.

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

fn invalid_input(message: &str) -> crate::Error {
	crate::Error::User(crate::UserError::ValidationFailed(message.to_owned()))
}

fn parse_h256(input: impl Into<HashString>) -> Result<H256, crate::Error> {
	let value: HashString = input.into();
	value
		.try_into()
		.map_err(|_| invalid_input("Malformed string is passed for H256"))
}

fn parse_multi_address(input: impl Into<MultiAddressLike>) -> Result<MultiAddress, crate::Error> {
	let value: MultiAddressLike = input.into();
	value
		.try_into()
		.map_err(|_| invalid_input("Malformed string is passed for AccountId"))
}

fn parse_account_id(input: impl Into<AccountIdLike>) -> Result<AccountId, crate::Error> {
	let value: AccountIdLike = input.into();
	value
		.try_into()
		.map_err(|_| invalid_input("Malformed string is passed for AccountId"))
}

fn parse_account_ids(values: Vec<impl Into<AccountIdLike>>) -> Result<Vec<AccountId>, crate::Error> {
	values.into_iter().map(parse_account_id).collect::<Result<Vec<_>, _>>()
}

fn parse_multi_addresses(values: Vec<impl Into<MultiAddressLike>>) -> Result<Vec<MultiAddress>, crate::Error> {
	values
		.into_iter()
		.map(parse_multi_address)
		.collect::<Result<Vec<_>, _>>()
}

/// Entry point for constructing pallet-specific transaction builders.
///
/// Each accessor clones the underlying [`Client`] and returns a lightweight helper that can compose
/// extrinsics without contacting the node. The returned builders produce [`SubmittableTransaction`]s
/// which must be signed—and optionally submitted—separately.
pub struct TransactionApi(pub(crate) Client);
impl TransactionApi {
	/// Returns helpers for composing `balances` pallet extrinsics.
	///
	/// Returns a [`Balances`] builder that clones this client.
	pub fn balances(&self) -> Balances {
		Balances(self.0.clone())
	}

	/// Returns helpers for composing data availability submissions.
	///
	/// Returns a [`DataAvailability`] builder that clones this client.
	pub fn data_availability(&self) -> DataAvailability {
		DataAvailability(self.0.clone())
	}

	/// Returns helpers for multisig transaction approval flows.
	///
	/// Returns a [`Multisig`] builder that clones this client.
	pub fn multisig(&self) -> Multisig {
		Multisig(self.0.clone())
	}

	/// Returns helpers for batching extrinsics via the utility pallet.
	///
	/// Returns a [`Utility`] builder that clones this client.
	pub fn utility(&self) -> Utility {
		Utility(self.0.clone())
	}

	/// Returns helpers for proxy management extrinsics.
	///
	/// Returns a [`Proxy`] builder that clones this client.
	pub fn proxy(&self) -> Proxy {
		Proxy(self.0.clone())
	}

	/// Returns helpers for staking-related extrinsics.
	///
	/// Returns a [`Staking`] builder that clones this client.
	pub fn staking(&self) -> Staking {
		Staking(self.0.clone())
	}

	/// Returns helpers for Vector message passing extrinsics.
	///
	/// Returns a [`Vector`] builder that clones this client.
	pub fn vector(&self) -> Vector {
		Vector(self.0.clone())
	}

	/// Returns helpers for system-level extrinsics.
	///
	/// Returns a [`System`] builder that clones this client.
	pub fn system(&self) -> System {
		System(self.0.clone())
	}

	/// Returns helpers for nomination pool extrinsics.
	///
	/// Returns a [`NominationPools`] builder that clones this client.
	pub fn nomination_pools(&self) -> NominationPools {
		NominationPools(self.0.clone())
	}

	/// Returns helpers for validator session key management.
	///
	/// Returns a [`Session`] builder that clones this client.
	pub fn session(&self) -> Session {
		Session(self.0.clone())
	}
}

/// Builds extrinsics for the `session` pallet.
///
/// The helper clones the underlying client; composing calls does not contact the node until the
/// resulting [`SubmittableTransaction`] is signed or submitted.
pub struct Session(Client);
impl Session {
	/// Updates the node's session keys with new authorities and proof data.
	///
	/// # Panics
	/// Panics when any supplied key fails to decode into an `H256` hash.
	///
	pub fn set_key(
		&self,
		babe: impl Into<HashString>,
		grandpa: impl Into<HashString>,
		authority_discovery: impl Into<HashString>,
		im_online: impl Into<HashString>,
		proof: Vec<u8>,
	) -> Result<SubmittableTransaction, crate::Error> {
		let babe = parse_h256(babe)?;
		let grandpa = parse_h256(grandpa)?;
		let authority_discovery = parse_h256(authority_discovery)?;
		let im_online = parse_h256(im_online)?;

		let value = avail::session::tx::SetKeys { babe, grandpa, authority_discovery, im_online, proof };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Removes the stored session keys from on-chain storage.
	///
	pub fn purge_key(&self) -> SubmittableTransaction {
		let value = avail::session::tx::PurgeKeys {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

/// Builds extrinsics for the `nomination_pools` pallet.
///
/// Many helpers accept `MultiAddressLike` values and will panic if those cannot be converted into
/// on-chain account identifiers. Constructing the [`SubmittableTransaction`] itself does not hit the
/// network; signing or submitting it will.
pub struct NominationPools(Client);
impl NominationPools {
	/// Contributes additional stake from the pool's bonded account.
	///
	pub fn bond_extra(&self, value: BondExtraValue) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::BondExtra { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Bonds additional stake on behalf of another member.
	///
	/// # Panics
	/// Panics if `member` cannot be converted into a `MultiAddress`.
	///
	pub fn bond_extra_other(
		&self,
		member: impl Into<MultiAddressLike>,
		value: BondExtraValue,
	) -> Result<SubmittableTransaction, crate::Error> {
		let member = parse_multi_address(member)?;

		let value = avail::nomination_pools::tx::BondExtraOther { member, value };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Requests the pool to chill its nominations.
	///
	pub fn chill(&self, pool_id: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::Chill { pool_id };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Claims pending commission for the given pool.
	///
	pub fn claim_commission(&self, pool_id: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::ClaimCommission { pool_id };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Claims a pending payout for the caller.
	///
	pub fn claim_payout(&self) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::ClaimPayout {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Claims a pending payout for another pool member.
	///
	/// # Panics
	/// Panics if `owner` cannot be converted into an `AccountId`.
	///
	pub fn claim_payout_other(&self, owner: impl Into<AccountIdLike>) -> Result<SubmittableTransaction, crate::Error> {
		let owner = parse_account_id(owner)?;

		let value = avail::nomination_pools::tx::ClaimPayoutOther { owner };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Creates a new nomination pool with freshly provided roles.
	///
	/// # Panics
	/// Panics if any of `root`, `nominator`, or `bouncer` cannot be converted into a `MultiAddress`.
	///
	pub fn create(
		&self,
		amount: u128,
		root: impl Into<MultiAddressLike>,
		nominator: impl Into<MultiAddressLike>,
		bouncer: impl Into<MultiAddressLike>,
	) -> Result<SubmittableTransaction, crate::Error> {
		let root = parse_multi_address(root)?;
		let nominator = parse_multi_address(nominator)?;
		let bouncer = parse_multi_address(bouncer)?;

		let value = avail::nomination_pools::tx::Create { amount, root, nominator, bouncer };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Creates a new nomination pool using a specific pool identifier.
	///
	/// # Panics
	/// Panics if any of `root`, `nominator`, or `bouncer` cannot be converted into a `MultiAddress`.
	///
	pub fn create_with_pool_id(
		&self,
		amount: u128,
		root: impl Into<MultiAddressLike>,
		nominator: impl Into<MultiAddressLike>,
		bouncer: impl Into<MultiAddressLike>,
		pool_id: u32,
	) -> Result<SubmittableTransaction, crate::Error> {
		let root = parse_multi_address(root)?;
		let nominator = parse_multi_address(nominator)?;
		let bouncer = parse_multi_address(bouncer)?;

		let value = avail::nomination_pools::tx::CreateWithPoolId { amount, root, nominator, bouncer, pool_id };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Joins an existing pool by contributing the requested amount.
	///
	pub fn join(&self, amount: u128, pool_id: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::Join { amount, pool_id };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Sets nominations for the pool to a new validator set.
	///
	/// # Panics
	/// Panics if any validator identifier cannot be converted into an `AccountId`.
	///
	pub fn nominate(
		&self,
		pool_id: u32,
		validators: Vec<impl Into<AccountIdLike>>,
	) -> Result<SubmittableTransaction, crate::Error> {
		let validators = parse_account_ids(validators)?;

		let value = avail::nomination_pools::tx::Nominate { pool_id, validators };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Updates who is allowed to claim rewards for the pool.
	///
	pub fn set_claim_permission(&self, permission: ClaimPermission) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetClaimPermission { permission };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the commission settings for the pool, optionally setting a payee.
	///
	/// # Panics
	/// Panics if the payee provided in `new_commission` cannot be converted into an `AccountId`.
	///
	pub fn set_commission(
		&self,
		pool_id: u32,
		new_commission: Option<(u32, AccountIdLike)>,
	) -> Result<SubmittableTransaction, crate::Error> {
		let new_commission = match new_commission {
			Some((commission, payee)) => Some((commission, parse_account_id(payee)?)),
			None => None,
		};
		let value = avail::nomination_pools::tx::SetCommission { pool_id, new_commission };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Configures how frequently pool commission may change.
	///
	pub fn set_commission_change_rate(
		&self,
		pool_id: u32,
		max_increase: u32,
		min_delay: u32,
	) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetCommissionChangeRate { pool_id, max_increase, min_delay };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Caps commission at the provided maximum percentage.
	///
	pub fn set_commission_max(&self, pool_id: u32, max_commission: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetCommissionMax { pool_id, max_commission };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates pool metadata stored on chain.
	///
	pub fn set_metadata<'a>(&self, pool_id: u32, metadata: impl Into<StringOrBytes<'a>>) -> SubmittableTransaction {
		let metadata: StringOrBytes = metadata.into();
		let metadata: Vec<u8> = metadata.into();
		let value = avail::nomination_pools::tx::SetMetadata { pool_id, metadata };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Transitions the pool into a new lifecycle state.
	///
	pub fn set_state(&self, pool_id: u32, state: PoolState) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetState { pool_id, state };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Starts the unbonding process for the specified member account.
	///
	/// # Panics
	/// Panics if `member_account` cannot be converted into a `MultiAddress`.
	///
	pub fn unbond(
		&self,
		member_account: impl Into<MultiAddressLike>,
		unbonding_points: u128,
	) -> Result<SubmittableTransaction, crate::Error> {
		let member_account = parse_multi_address(member_account)?;

		let value = avail::nomination_pools::tx::Unbond { member_account, unbonding_points };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Updates the pool's root, nominator, and bouncer roles.
	///
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

	/// Withdraws fully unbonded funds for the given member account.
	///
	/// # Panics
	/// Panics if `member_account` cannot be converted into a `MultiAddress`.
	///
	pub fn withdraw_unbonded(
		&self,
		member_account: impl Into<MultiAddressLike>,
		num_slashing_spans: u32,
	) -> Result<SubmittableTransaction, crate::Error> {
		let member_account = parse_multi_address(member_account)?;

		let value = avail::nomination_pools::tx::WithdrawUnbonded { member_account, num_slashing_spans };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}
}

/// Builds extrinsics for the `staking` pallet.
///
/// Methods that accept `AccountIdLike` or `MultiAddressLike` parameters will panic if the provided
/// value cannot be converted into the expected on-chain representation.
pub struct Staking(Client);
impl Staking {
	/// Bonds funds from the controller with the provided reward destination.
	///
	pub fn bond(&self, value: u128, payee: RewardDestination) -> SubmittableTransaction {
		let value = avail::staking::tx::Bond { value, payee };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Adds additional stake on top of an existing bond.
	///
	pub fn bond_extra(&self, value: u128) -> SubmittableTransaction {
		let value = avail::staking::tx::BondExtra { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Starts unbonding the given amount of funds.
	///
	pub fn unbond(&self, value: u128) -> SubmittableTransaction {
		let value = avail::staking::tx::Unbond { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Re-bonds a portion of funds that are currently unbonding.
	///
	pub fn rebond(&self, value: u128) -> SubmittableTransaction {
		let value = avail::staking::tx::Rebond { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Advertises validator preferences for the caller.
	///
	pub fn validate(&self, commission: u32, blocked: bool) -> SubmittableTransaction {
		let value = avail::staking::tx::Validate { prefs: ValidatorPrefs { commission, blocked } };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Nominates a new set of validator targets.
	///
	/// # Panics
	/// Panics if any provided target cannot be converted into a `MultiAddress`.
	///
	pub fn nominate(&self, targets: Vec<impl Into<MultiAddressLike>>) -> Result<SubmittableTransaction, crate::Error> {
		let targets = parse_multi_addresses(targets)?;

		let value = avail::staking::tx::Nominate { targets };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Pays out staking rewards for the given validator and era.
	///
	/// # Panics
	/// Panics if `validator_stash` cannot be converted into an `AccountId`.
	///
	pub fn payout_stakers(
		&self,
		validator_stash: impl Into<AccountIdLike>,
		era: u32,
	) -> Result<SubmittableTransaction, crate::Error> {
		let validator_stash = parse_account_id(validator_stash)?;

		let value = avail::staking::tx::PayoutStakers { validator_stash, era };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Switches the controller account for the stash.
	///
	pub fn set_controller(&self) -> SubmittableTransaction {
		let value = avail::staking::tx::SetController {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the staking reward destination.
	///
	pub fn set_payee(&self, payee: RewardDestination) -> SubmittableTransaction {
		let value = avail::staking::tx::SetPayee { payee };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Stops nominating for the caller.
	///
	pub fn chill(&self) -> SubmittableTransaction {
		let value = avail::staking::tx::Chill {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Issues a chill for another stash account.
	///
	/// # Panics
	/// Panics if `stash` cannot be converted into an `AccountId`.
	///
	pub fn chill_other(&self, stash: impl Into<AccountIdLike>) -> Result<SubmittableTransaction, crate::Error> {
		let stash = parse_account_id(stash)?;

		let value = avail::staking::tx::ChillOther { stash };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Withdraws funds that have completed the unbonding period.
	///
	pub fn withdraw_unbonded(&self, num_slashing_spans: u32) -> SubmittableTransaction {
		let value = avail::staking::tx::WithdrawUnbonded { num_slashing_spans };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Removes a stash that no longer has bonded funds.
	///
	/// # Panics
	/// Panics if `stash` cannot be converted into an `AccountId`.
	///
	pub fn reap_stash(
		&self,
		stash: impl Into<AccountIdLike>,
		num_slashing_spans: u32,
	) -> Result<SubmittableTransaction, crate::Error> {
		let stash = parse_account_id(stash)?;

		let value = avail::staking::tx::ReapStash { stash, num_slashing_spans };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Removes the provided nominees from the caller's nomination list.
	///
	/// # Panics
	/// Panics if any identifier in `who` cannot be converted into a `MultiAddress`.
	///
	pub fn kick(&self, who: Vec<impl Into<MultiAddressLike>>) -> Result<SubmittableTransaction, crate::Error> {
		let who = parse_multi_addresses(who)?;

		let value = avail::staking::tx::Kick { who };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Forces the commission for the given validator to the chain minimum.
	///
	/// # Panics
	/// Panics if `validator_stash` cannot be converted into an `AccountId`.
	///
	pub fn force_apply_min_commission(
		&self,
		validator_stash: impl Into<AccountIdLike>,
	) -> Result<SubmittableTransaction, crate::Error> {
		let validator_stash = parse_account_id(validator_stash)?;

		let value = avail::staking::tx::ForceApplyMinCommission { validator_stash };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Pays out staking rewards for a subset of nominators.
	///
	/// # Panics
	/// Panics if `validator_stash` cannot be converted into an `AccountId`.
	///
	pub fn payout_stakers_by_page(
		&self,
		validator_stash: impl Into<AccountIdLike>,
		era: u32,
		page: u32,
	) -> Result<SubmittableTransaction, crate::Error> {
		let validator_stash = parse_account_id(validator_stash)?;

		let value = avail::staking::tx::PayoutStakersByPage { validator_stash, era, page };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}
}

/// Builds extrinsics for the `balances` pallet.
///
/// All helpers expect account identifiers that can be converted into `MultiAddress` values and will
/// panic if the conversion fails.
pub struct Balances(Client);
impl Balances {
	/// Transfers funds allowing the sender's account to be removed if depleted.
	///
	/// # Panics
	/// Panics if `dest` cannot be converted into a `MultiAddress`.
	///
	pub fn transfer_allow_death(
		&self,
		dest: impl Into<MultiAddressLike>,
		amount: u128,
	) -> Result<SubmittableTransaction, crate::Error> {
		let dest = parse_multi_address(dest)?;

		let value = avail::balances::tx::TransferAllowDeath { dest, value: amount };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Transfers funds while keeping the sender's account alive.
	///
	/// # Panics
	/// Panics if `dest` cannot be converted into a `MultiAddress`.
	///
	pub fn transfer_keep_alive(
		&self,
		dest: impl Into<MultiAddressLike>,
		amount: u128,
	) -> Result<SubmittableTransaction, crate::Error> {
		let dest = parse_multi_address(dest)?;

		let value = avail::balances::tx::TransferKeepAlive { dest, value: amount };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Transfers the entire free balance to the destination.
	///
	/// # Panics
	/// Panics if `dest` cannot be converted into a `MultiAddress`.
	///
	pub fn transfer_all(
		&self,
		dest: impl Into<MultiAddressLike>,
		keep_alive: bool,
	) -> Result<SubmittableTransaction, crate::Error> {
		let dest = parse_multi_address(dest)?;

		let value = avail::balances::tx::TransferAll { dest, keep_alive };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}
}

/// Builds extrinsics for the `multisig` pallet.
///
/// Helper methods convert `AccountIdLike` and `HashString` inputs into on-chain representations and
/// panic if the conversion fails; they also sort the provided signatories to match runtime
/// expectations.
pub struct Multisig(Client);
impl Multisig {
	/// Approves a multisig call by reference to its hash.
	///
	/// # Panics
	/// Panics if any signatory identifier fails to convert into an `AccountId` or if `call_hash`
	/// cannot be converted into `H256`.
	///
	pub fn approve_as_multi(
		&self,
		threshold: u16,
		other_signatories: Vec<impl Into<AccountIdLike>>,
		maybe_timepoint: Option<Timepoint>,
		call_hash: impl Into<HashString>,
		max_weight: Weight,
	) -> Result<SubmittableTransaction, crate::Error> {
		fn inner(
			client: Client,
			threshold: u16,
			other_signatories: Vec<AccountIdLike>,
			maybe_timepoint: Option<Timepoint>,
			call_hash: HashString,
			max_weight: Weight,
		) -> Result<SubmittableTransaction, crate::Error> {
			let other_signatories: Result<Vec<AccountId>, _> =
				other_signatories.into_iter().map(|x| x.try_into()).collect();
			let mut other_signatories =
				other_signatories.map_err(|_| invalid_input("Malformed string is passed for AccountId"))?;
			other_signatories.sort();

			let call_hash: H256 = call_hash
				.try_into()
				.map_err(|_| invalid_input("Malformed string is passed for H256"))?;

			let value = avail::multisig::tx::ApproveAsMulti {
				threshold,
				other_signatories,
				maybe_timepoint,
				call_hash,
				max_weight,
			};
			Ok(SubmittableTransaction::from_encodable(client, value))
		}

		let other_signatories: Vec<AccountIdLike> = other_signatories.into_iter().map(|x| x.into()).collect();
		let call_hash: HashString = call_hash.into();
		inner(self.0.clone(), threshold, other_signatories, maybe_timepoint, call_hash, max_weight)
	}

	/// Executes a multisig call with full call data.
	///
	/// # Panics
	/// Panics if any signatory identifier fails to convert into an `AccountId`.
	///
	pub fn as_multi(
		&self,
		threshold: u16,
		other_signatories: Vec<impl Into<AccountIdLike>>,
		maybe_timepoint: Option<Timepoint>,
		call: impl Into<ExtrinsicCall>,
		max_weight: Weight,
	) -> Result<SubmittableTransaction, crate::Error> {
		fn inner(
			client: Client,
			threshold: u16,
			other_signatories: Vec<AccountIdLike>,
			maybe_timepoint: Option<Timepoint>,
			call: ExtrinsicCall,
			max_weight: Weight,
		) -> Result<SubmittableTransaction, crate::Error> {
			let other_signatories: Result<Vec<AccountId>, _> =
				other_signatories.into_iter().map(|x| x.try_into()).collect();
			let mut other_signatories =
				other_signatories.map_err(|_| invalid_input("Malformed string is passed for AccountId"))?;
			other_signatories.sort();

			let value = avail::multisig::tx::AsMulti {
				threshold,
				other_signatories,
				maybe_timepoint,
				call,
				max_weight,
			};
			Ok(SubmittableTransaction::from_encodable(client, value))
		}

		let other_signatories: Vec<AccountIdLike> = other_signatories.into_iter().map(|x| x.into()).collect();
		inner(self.0.clone(), threshold, other_signatories, maybe_timepoint, call.into(), max_weight)
	}

	/// Executes a multisig call with a threshold of one.
	///
	/// # Panics
	/// Panics if any signatory identifier fails to convert into an `AccountId`.
	///
	pub fn as_multi_threshold_1(
		&self,
		other_signatories: Vec<impl Into<AccountIdLike>>,
		call: impl Into<ExtrinsicCall>,
	) -> Result<SubmittableTransaction, crate::Error> {
		fn inner(
			client: Client,
			other_signatories: Vec<AccountIdLike>,
			call: ExtrinsicCall,
		) -> Result<SubmittableTransaction, crate::Error> {
			let other_signatories: Result<Vec<AccountId>, _> =
				other_signatories.into_iter().map(|x| x.try_into()).collect();
			let mut other_signatories =
				other_signatories.map_err(|_| invalid_input("Malformed string is passed for AccountId"))?;
			other_signatories.sort();

			let value = avail::multisig::tx::AsMultiThreshold1 { other_signatories, call };
			Ok(SubmittableTransaction::from_encodable(client, value))
		}

		let other_signatories: Vec<AccountIdLike> = other_signatories.into_iter().map(|x| x.into()).collect();
		inner(self.0.clone(), other_signatories, call.into())
	}

	/// Cancels a previously scheduled multisig call.
	///
	/// # Panics
	/// Panics if any signatory identifier fails to convert into an `AccountId` or if `call_hash`
	/// cannot be converted into `H256`.
	///
	pub fn cancel_as_multi(
		&self,
		threshold: u16,
		other_signatories: Vec<impl Into<AccountIdLike>>,
		timepoint: Timepoint,
		call_hash: impl Into<HashString>,
	) -> Result<SubmittableTransaction, crate::Error> {
		fn inner(
			client: Client,
			threshold: u16,
			other_signatories: Vec<AccountIdLike>,
			timepoint: Timepoint,
			call_hash: HashString,
		) -> Result<SubmittableTransaction, crate::Error> {
			let other_signatories: Result<Vec<AccountId>, _> =
				other_signatories.into_iter().map(|x| x.try_into()).collect();
			let mut other_signatories =
				other_signatories.map_err(|_| invalid_input("Malformed string is passed for AccountId"))?;
			other_signatories.sort();

			let call_hash: H256 = call_hash
				.try_into()
				.map_err(|_| invalid_input("Malformed string is passed for H256"))?;

			let value = avail::multisig::tx::CancelAsMulti { threshold, other_signatories, timepoint, call_hash };
			Ok(SubmittableTransaction::from_encodable(client, value))
		}

		let other_signatories: Vec<AccountIdLike> = other_signatories.into_iter().map(|x| x.into()).collect();
		let call_hash: HashString = call_hash.into();
		inner(self.0.clone(), threshold, other_signatories, timepoint, call_hash)
	}
}

/// Builds extrinsics for the `data_availability` pallet.
pub struct DataAvailability(Client);
impl DataAvailability {
	/// Registers a new application key for data availability submissions.
	///
	pub fn create_application_key<'a>(&self, key: impl Into<StringOrBytes<'a>>) -> SubmittableTransaction {
		let key: Vec<u8> = Into::<StringOrBytes>::into(key).into();
		let value = avail::data_availability::tx::CreateApplicationKey { key };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Submits application data for availability guarantees.
	///
	pub fn submit_data<'a>(&self, app_id: u32, data: impl Into<StringOrBytes<'a>>) -> SubmittableTransaction {
		let data: Vec<u8> = Into::<StringOrBytes>::into(data).into();
		let value = avail::data_availability::tx::SubmitData { app_id, data };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Submits metadata describing an out-of-band blob.
	///
	pub fn submit_blob_metadata(
		&self,
		app_id: u32,
		blob_hash: H256,
		size: u64,
		commitments: Vec<u8>,
		eval_point_seed: Option<[u8; 32]>,
		eval_claim: Option<[u8; 16]>,
	) -> SubmittableTransaction {
		let value = avail::data_availability::tx::SubmitBlobMetadata {
			app_id,
			blob_hash,
			size,
			commitments,
			eval_point_seed,
			eval_claim,
		};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

/// Builds extrinsics for the `utility` pallet.
pub struct Utility(Client);
impl Utility {
	/// Dispatches a set of calls sequentially, aborting on failure.
	///
	pub fn batch(&self, calls: Vec<impl Into<ExtrinsicCall>>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::Batch::new();
		batch.add_calls(calls.into_iter().map(|x| x.into()).collect());
		SubmittableTransaction::from_encodable(self.0.clone(), batch)
	}

	/// Dispatches a set of calls and reverts the whole batch if any fail.
	///
	pub fn batch_all(&self, calls: Vec<impl Into<ExtrinsicCall>>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::BatchAll::new();
		batch.add_calls(calls.into_iter().map(|x| x.into()).collect());
		SubmittableTransaction::from_encodable(self.0.clone(), batch)
	}

	/// Dispatches a set of calls while ignoring failures.
	///
	pub fn force_batch(&self, calls: Vec<impl Into<ExtrinsicCall>>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::ForceBatch::new();
		batch.add_calls(calls.into_iter().map(|x| x.into()).collect());
		SubmittableTransaction::from_encodable(self.0.clone(), batch)
	}
}

/// Builds extrinsics for the `proxy` pallet.
///
/// Methods converting `MultiAddressLike` parameters will panic if the provided values cannot be
/// decoded into `MultiAddress` instances.
pub struct Proxy(Client);
impl Proxy {
	/// Dispatches a call through an existing proxy relationship.
	///
	/// # Panics
	/// Panics if `id` cannot be converted into a `MultiAddress`.
	///
	pub fn proxy(
		&self,
		id: impl Into<MultiAddressLike>,
		force_proxy_type: Option<ProxyType>,
		call: impl Into<ExtrinsicCall>,
	) -> Result<SubmittableTransaction, crate::Error> {
		let id = parse_multi_address(id)?;

		let value = avail::proxy::tx::Proxy { id, force_proxy_type, call: call.into() };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Registers a new proxy delegate for the caller.
	///
	/// # Panics
	/// Panics if `id` cannot be converted into a `MultiAddress`.
	///
	pub fn add_proxy(
		&self,
		id: impl Into<MultiAddressLike>,
		proxy_type: ProxyType,
		delay: u32,
	) -> Result<SubmittableTransaction, crate::Error> {
		let id = parse_multi_address(id)?;

		let value = avail::proxy::tx::AddProxy { id, proxy_type, delay };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Removes a specific proxy delegate.
	///
	/// # Panics
	/// Panics if `delegate` cannot be converted into a `MultiAddress`.
	///
	pub fn remove_proxy(
		&self,
		delegate: impl Into<MultiAddressLike>,
		proxy_type: ProxyType,
		delay: u32,
	) -> Result<SubmittableTransaction, crate::Error> {
		let delegate = parse_multi_address(delegate)?;

		let value = avail::proxy::tx::RemoveProxy { delegate, proxy_type, delay };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Removes all proxies belonging to the caller.
	///
	pub fn remove_proxies(&self) -> SubmittableTransaction {
		let value = avail::proxy::tx::RemoveProxies {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Creates a pure proxy account with the requested parameters.
	///
	pub fn create_pure(&self, proxy_type: ProxyType, delay: u32, index: u16) -> SubmittableTransaction {
		let value = avail::proxy::tx::CreatePure { proxy_type, delay, index };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Kills a pure proxy that was previously spawned by the provided account.
	///
	/// # Panics
	/// Panics if `spawner` cannot be converted into a `MultiAddress`.
	///
	pub fn kill_pure(
		&self,
		spawner: impl Into<MultiAddressLike>,
		proxy_type: ProxyType,
		index: u16,
		height: u32,
		ext_index: u32,
	) -> Result<SubmittableTransaction, crate::Error> {
		let spawner = parse_multi_address(spawner)?;

		let value = avail::proxy::tx::KillPure { spawner, proxy_type, index, height, ext_index };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}
}

/// Builds extrinsics for the `vector` pallet.
///
/// Several helpers convert hash-like parameters into `H256` values and will panic if the provided
/// data cannot be parsed.
pub struct Vector(Client);
impl Vector {
	/// Submits a fulfillment proof for a pending cross-chain call.
	///
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

	/// Executes a vector addressed message with witness data.
	///
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

	/// Toggles whether a source chain is frozen.
	///
	pub fn source_chain_froze(&self, source_chain_id: u32, frozen: bool) -> SubmittableTransaction {
		let value = avail::vector::tx::SourceChainFroze { source_chain_id, frozen };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Sends a vector message to the specified domain.
	///
	/// # Panics
	/// Panics if `to` cannot be converted into an `H256`.
	///
	pub fn send_message(
		&self,
		message: avail::vector::types::Message,
		to: impl Into<HashString>,
		domain: u32,
	) -> Result<SubmittableTransaction, crate::Error> {
		let to = parse_h256(to)?;

		let value = avail::vector::tx::SendMessage { message, to, domain };
		Ok(SubmittableTransaction::from_encodable(self.0.clone(), value))
	}

	/// Marks previous outbound messages as failed by index.
	///
	pub fn failed_send_message_txs(&self, failed_txs: Vec<u32>) -> SubmittableTransaction {
		let value = avail::vector::tx::FailedSendMessageTxs { failed_txs };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the Poseidon hash commitment for a sync period.
	///
	pub fn set_poseidon_hash(&self, period: u64, poseidon_hash: Vec<u8>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetPoseidonHash { period: period.into(), poseidon_hash };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Registers the broadcaster for a specific domain.
	///
	pub fn set_broadcaster(&self, broadcaster_domain: u32, broadcaster: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetBroadcaster { broadcaster_domain: broadcaster_domain.into(), broadcaster };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Overwrites the set of domains allowed to send messages.
	///
	pub fn set_whitelisted_domains(&self, value: Vec<u32>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetWhitelistedDomains { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the vector configuration parameters.
	///
	pub fn set_configuration(&self, value: avail::vector::types::Configuration) -> SubmittableTransaction {
		let value = avail::vector::tx::SetConfiguration { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the function identifiers used by the pallet.
	///
	pub fn set_function_ids(&self, value: Option<(H256, H256)>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetFunctionIds { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Sets the verification key for the step circuit.
	///
	pub fn set_step_verification_key(&self, value: Option<Vec<u8>>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetStepVerificationKey { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the updater account hash.
	///
	pub fn set_updater(&self, updater: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetUpdater { updater };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Submits a zero-knowledge proof fulfilling a pending message.
	///
	pub fn fulfill(&self, proof: Vec<u8>, public_values: Vec<u8>) -> SubmittableTransaction {
		let value = avail::vector::tx::Fulfill { proof, public_values };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Sets the verification key for SP1 proofs.
	///
	pub fn set_sp1_verification_key(&self, sp1_vk: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetSp1VerificationKey { sp1_vk };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the sync committee hash for the provided period.
	///
	pub fn set_sync_committee_hash(&self, period: u64, hash: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetSyncCommitteeHash { period, hash };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Enables or disables mock execution mode.
	///
	pub fn enable_mock(&self, value: bool) -> SubmittableTransaction {
		let value = avail::vector::tx::EnableMock { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Fulfills a message when running in mock mode.
	///
	pub fn mock_fulfill(&self, public_values: Vec<u8>) -> SubmittableTransaction {
		let value = avail::vector::tx::MockFulfill { public_values };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

/// Builds extrinsics for the `system` pallet.
pub struct System(Client);
impl System {
	/// Emits a remark event containing arbitrary bytes.
	///
	pub fn remark(&self, remark: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::Remark { remark };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Replaces the runtime code with a new version.
	///
	pub fn set_code(&self, code: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::SetCode { code };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Replaces the runtime code without performing standard checks.
	///
	pub fn set_code_without_checks(&self, code: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::SetCodeWithoutChecks { code };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Emits a remark while guaranteeing an event is produced.
	///
	pub fn remark_with_event(&self, remark: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::RemarkWithEvent { remark };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}
