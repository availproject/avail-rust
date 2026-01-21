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

/// Entry point for constructing pallet-specific transaction builders.
///
/// Each accessor clones the underlying [`Client`] and returns a lightweight helper that can compose
/// extrinsics without contacting the node. The returned builders produce [`SubmittableTransaction`]s
/// which must be signed—and optionally submitted—separately.
pub struct TransactionApi(pub(crate) Client);
impl TransactionApi {
	/// Returns helpers for composing `balances` pallet extrinsics.
	///
	/// # Returns
	/// Returns a [`Balances`] builder that clones this client.
	pub fn balances(&self) -> Balances {
		Balances(self.0.clone())
	}

	/// Returns helpers for composing data availability submissions.
	///
	/// # Returns
	/// Returns a [`DataAvailability`] builder that clones this client.
	pub fn data_availability(&self) -> DataAvailability {
		DataAvailability(self.0.clone())
	}

	/// Returns helpers for multisig transaction approval flows.
	///
	/// # Returns
	/// Returns a [`Multisig`] builder that clones this client.
	pub fn multisig(&self) -> Multisig {
		Multisig(self.0.clone())
	}

	/// Returns helpers for batching extrinsics via the utility pallet.
	///
	/// # Returns
	/// Returns a [`Utility`] builder that clones this client.
	pub fn utility(&self) -> Utility {
		Utility(self.0.clone())
	}

	/// Returns helpers for proxy management extrinsics.
	///
	/// # Returns
	/// Returns a [`Proxy`] builder that clones this client.
	pub fn proxy(&self) -> Proxy {
		Proxy(self.0.clone())
	}

	/// Returns helpers for staking-related extrinsics.
	///
	/// # Returns
	/// Returns a [`Staking`] builder that clones this client.
	pub fn staking(&self) -> Staking {
		Staking(self.0.clone())
	}

	/// Returns helpers for Vector message passing extrinsics.
	///
	/// # Returns
	/// Returns a [`Vector`] builder that clones this client.
	pub fn vector(&self) -> Vector {
		Vector(self.0.clone())
	}

	/// Returns helpers for system-level extrinsics.
	///
	/// # Returns
	/// Returns a [`System`] builder that clones this client.
	pub fn system(&self) -> System {
		System(self.0.clone())
	}

	/// Returns helpers for nomination pool extrinsics.
	///
	/// # Returns
	/// Returns a [`NominationPools`] builder that clones this client.
	pub fn nomination_pools(&self) -> NominationPools {
		NominationPools(self.0.clone())
	}

	/// Returns helpers for validator session key management.
	///
	/// # Returns
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
	/// # Arguments
	/// * `babe` - BABE authority key encoded as a hash string.
	/// * `grandpa` - GRANDPA authority key encoded as a hash string.
	/// * `authority_discovery` - Authority discovery key encoded as a hash string.
	/// * `im_online` - Im-online session key encoded as a hash string.
	/// * `proof` - Proof bytes returned by the session key generator.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that sets the supplied session keys.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

	/// Removes the stored session keys from on-chain storage.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that clears session keys for the signing account.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
	/// # Arguments
	/// * `value` - Amount to bond, expressed as a [`BondExtraValue`].
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that bonds the extra amount for the pool.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn bond_extra(&self, value: BondExtraValue) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::BondExtra { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Bonds additional stake on behalf of another member.
	///
	/// # Panics
	/// Panics if `member` cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `member` - Account that receives the increased bonded amount.
	/// * `value` - Amount to bond, expressed as a [`BondExtraValue`].
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that bonds extra stake for the specified member.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

	/// Requests the pool to chill its nominations.
	///
	/// # Arguments
	/// * `pool_id` - Identifier of the pool that should chill.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that issues the `chill` request.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn chill(&self, pool_id: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::Chill { pool_id };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Claims pending commission for the given pool.
	///
	/// # Arguments
	/// * `pool_id` - Identifier of the pool that should pay out commission.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that claims the commission.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn claim_commission(&self, pool_id: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::ClaimCommission { pool_id };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Claims a pending payout for the caller.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that claims unpaid rewards for the signer.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn claim_payout(&self) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::ClaimPayout {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Claims a pending payout for another pool member.
	///
	/// # Panics
	/// Panics if `owner` cannot be converted into an `AccountId`.
	///
	/// # Arguments
	/// * `owner` - Account that receives the payout.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that claims unpaid rewards on behalf of `owner`.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn claim_payout_other(&self, owner: impl Into<AccountIdLike>) -> SubmittableTransaction {
		let owner: AccountIdLike = owner.into();
		let owner: AccountId = owner.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::nomination_pools::tx::ClaimPayoutOther { owner };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Creates a new nomination pool with freshly provided roles.
	///
	/// # Panics
	/// Panics if any of `root`, `nominator`, or `bouncer` cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `amount` - Initial bonded amount for the pool.
	/// * `root` - Root account controlling pool administration.
	/// * `nominator` - Account authorised to nominate validators.
	/// * `bouncer` - Account that manages membership access.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that creates the pool with the supplied roles.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

	/// Creates a new nomination pool using a specific pool identifier.
	///
	/// # Panics
	/// Panics if any of `root`, `nominator`, or `bouncer` cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `amount` - Initial bonded amount for the pool.
	/// * `root` - Root account controlling pool administration.
	/// * `nominator` - Account authorised to nominate validators.
	/// * `bouncer` - Account that manages membership access.
	/// * `pool_id` - Identifier to assign to the new pool.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that creates the pool with an explicit identifier.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

	/// Joins an existing pool by contributing the requested amount.
	///
	/// # Arguments
	/// * `amount` - Amount of stake contributed by the caller.
	/// * `pool_id` - Identifier of the pool to join.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that adds the caller to the pool.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn join(&self, amount: u128, pool_id: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::Join { amount, pool_id };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Sets nominations for the pool to a new validator set.
	///
	/// # Panics
	/// Panics if any validator identifier cannot be converted into an `AccountId`.
	///
	/// # Arguments
	/// * `pool_id` - Identifier of the pool whose nominations are updated.
	/// * `validators` - Validators that the pool should nominate.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that updates the pool's nominations.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn nominate(&self, pool_id: u32, validators: Vec<impl Into<AccountIdLike>>) -> SubmittableTransaction {
		let validators: Vec<AccountIdLike> = validators.into_iter().map(|x| x.into()).collect();
		let validators: Result<Vec<AccountId>, _> = validators.into_iter().map(AccountId::try_from).collect();
		let validators = validators.expect("Malformed string is passed for AccountId");

		let value = avail::nomination_pools::tx::Nominate { pool_id, validators };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates who is allowed to claim rewards for the pool.
	///
	/// # Arguments
	/// * `permission` - Claim policy applied to the pool.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that adjusts the pool's claim permission.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_claim_permission(&self, permission: ClaimPermission) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetClaimPermission { permission };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the commission settings for the pool, optionally setting a payee.
	///
	/// # Panics
	/// Panics if the payee provided in `new_commission` cannot be converted into an `AccountId`.
	///
	/// # Arguments
	/// * `pool_id` - Identifier of the pool whose commission is updated.
	/// * `new_commission` - Optional tuple of `(commission, payee)` describing the new rate and payee.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that updates commission settings.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_commission(&self, pool_id: u32, new_commission: Option<(u32, AccountIdLike)>) -> SubmittableTransaction {
		let new_commission =
			new_commission.map(|x| (x.0, AccountId::try_from(x.1).expect("Malformed string is passed for AccountId")));
		let value = avail::nomination_pools::tx::SetCommission { pool_id, new_commission };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Configures how frequently pool commission may change.
	///
	/// # Arguments
	/// * `pool_id` - Identifier of the pool being updated.
	/// * `max_increase` - Maximum commission increase allowed per change.
	/// * `min_delay` - Minimum number of eras between commission updates.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that applies the new change rate.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
	/// # Arguments
	/// * `pool_id` - Identifier of the pool being updated.
	/// * `max_commission` - Maximum commission percentage allowed.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that stores the new commission cap.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_commission_max(&self, pool_id: u32, max_commission: u32) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetCommissionMax { pool_id, max_commission };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates pool metadata stored on chain.
	///
	/// # Arguments
	/// * `pool_id` - Identifier of the pool being updated.
	/// * `metadata` - Metadata payload encoded as bytes or string.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that writes the metadata to storage.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_metadata<'a>(&self, pool_id: u32, metadata: impl Into<StringOrBytes<'a>>) -> SubmittableTransaction {
		let metadata: StringOrBytes = metadata.into();
		let metadata: Vec<u8> = metadata.into();
		let value = avail::nomination_pools::tx::SetMetadata { pool_id, metadata };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Transitions the pool into a new lifecycle state.
	///
	/// # Arguments
	/// * `pool_id` - Identifier of the pool being updated.
	/// * `state` - New lifecycle state to apply.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that updates the pool state.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_state(&self, pool_id: u32, state: PoolState) -> SubmittableTransaction {
		let value = avail::nomination_pools::tx::SetState { pool_id, state };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Starts the unbonding process for the specified member account.
	///
	/// # Panics
	/// Panics if `member_account` cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `member_account` - Account leaving the pool.
	/// * `unbonding_points` - Amount of stake to unbond, expressed in pool points.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that schedules the unbonding.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

	/// Updates the pool's root, nominator, and bouncer roles.
	///
	/// # Arguments
	/// * `pool_id` - Identifier of the pool being updated.
	/// * `new_root` - Operation describing how to update the root account.
	/// * `new_nominator` - Operation describing how to update the nominator account.
	/// * `new_bouncer` - Operation describing how to update the bouncer account.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that applies the new role assignments.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
	/// # Arguments
	/// * `member_account` - Account withdrawing previously unbonded funds.
	/// * `num_slashing_spans` - Number of slashing spans to consider when finalising the withdrawal.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that withdraws the unbonded amount.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

/// Builds extrinsics for the `staking` pallet.
///
/// Methods that accept `AccountIdLike` or `MultiAddressLike` parameters will panic if the provided
/// value cannot be converted into the expected on-chain representation.
pub struct Staking(Client);
impl Staking {
	/// Bonds funds from the controller with the provided reward destination.
	///
	/// # Arguments
	/// * `value` - Amount of stake to bond.
	/// * `payee` - Destination where rewards should be paid.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that bonds the specified amount.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn bond(&self, value: u128, payee: RewardDestination) -> SubmittableTransaction {
		let value = avail::staking::tx::Bond { value, payee };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Adds additional stake on top of an existing bond.
	///
	/// # Arguments
	/// * `value` - Additional stake to add to the bonded balance.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that increases the bonded amount.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn bond_extra(&self, value: u128) -> SubmittableTransaction {
		let value = avail::staking::tx::BondExtra { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Starts unbonding the given amount of funds.
	///
	/// # Arguments
	/// * `value` - Amount of stake to unbond.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that schedules the unbonding.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn unbond(&self, value: u128) -> SubmittableTransaction {
		let value = avail::staking::tx::Unbond { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Re-bonds a portion of funds that are currently unbonding.
	///
	/// # Arguments
	/// * `value` - Amount of stake to re-bond.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that re-bonds the requested amount.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn rebond(&self, value: u128) -> SubmittableTransaction {
		let value = avail::staking::tx::Rebond { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Advertises validator preferences for the caller.
	///
	/// # Arguments
	/// * `commission` - Desired commission percentage.
	/// * `blocked` - Flag indicating whether new nominations are rejected.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that publishes the validator preferences.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn validate(&self, commission: u32, blocked: bool) -> SubmittableTransaction {
		let value = avail::staking::tx::Validate { prefs: ValidatorPrefs { commission, blocked } };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Nominates a new set of validator targets.
	///
	/// # Panics
	/// Panics if any provided target cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `targets` - Validators to nominate.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that updates the nomination targets.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn nominate(&self, targets: Vec<impl Into<MultiAddressLike>>) -> SubmittableTransaction {
		let targets: Vec<MultiAddressLike> = targets.into_iter().map(|x| x.into()).collect();
		let targets: Result<Vec<MultiAddress>, _> = targets.into_iter().map(MultiAddress::try_from).collect();
		let targets = targets.expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::Nominate { targets };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Pays out staking rewards for the given validator and era.
	///
	/// # Panics
	/// Panics if `validator_stash` cannot be converted into an `AccountId`.
	///
	/// # Arguments
	/// * `validator_stash` - Stash account whose rewards are claimed.
	/// * `era` - Era for which rewards are paid out.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that triggers the payout.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn payout_stakers(&self, validator_stash: impl Into<AccountIdLike>, era: u32) -> SubmittableTransaction {
		let validator_stash: AccountIdLike = validator_stash.into();
		let validator_stash = AccountId::try_from(validator_stash).expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::PayoutStakers { validator_stash, era };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Switches the controller account for the stash.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that sets a new controller (based on the signature).
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_controller(&self) -> SubmittableTransaction {
		let value = avail::staking::tx::SetController {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the staking reward destination.
	///
	/// # Arguments
	/// * `payee` - Destination where new rewards should be deposited.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that updates the reward destination.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_payee(&self, payee: RewardDestination) -> SubmittableTransaction {
		let value = avail::staking::tx::SetPayee { payee };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Stops nominating for the caller.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that chills the caller's nominations.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn chill(&self) -> SubmittableTransaction {
		let value = avail::staking::tx::Chill {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Issues a chill for another stash account.
	///
	/// # Panics
	/// Panics if `stash` cannot be converted into an `AccountId`.
	///
	/// # Arguments
	/// * `stash` - Stash account to chill.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that chills the specified stash.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn chill_other(&self, stash: impl Into<AccountIdLike>) -> SubmittableTransaction {
		let stash: AccountIdLike = stash.into();
		let stash = AccountId::try_from(stash).expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::ChillOther { stash };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Withdraws funds that have completed the unbonding period.
	///
	/// # Arguments
	/// * `num_slashing_spans` - Number of slashing spans to consider when finalising the withdrawal.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that withdraws matured unbonded funds.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn withdraw_unbonded(&self, num_slashing_spans: u32) -> SubmittableTransaction {
		let value = avail::staking::tx::WithdrawUnbonded { num_slashing_spans };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Removes a stash that no longer has bonded funds.
	///
	/// # Panics
	/// Panics if `stash` cannot be converted into an `AccountId`.
	///
	/// # Arguments
	/// * `stash` - Stash account to reap.
	/// * `num_slashing_spans` - Number of slashing spans considered during reaping.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that reaps the empty stash.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn reap_stash(&self, stash: impl Into<AccountIdLike>, num_slashing_spans: u32) -> SubmittableTransaction {
		let stash: AccountIdLike = stash.into();
		let stash = AccountId::try_from(stash).expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::ReapStash { stash, num_slashing_spans };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Removes the provided nominees from the caller's nomination list.
	///
	/// # Panics
	/// Panics if any identifier in `who` cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `who` - Nominees to remove.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that removes the specified nominees.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn kick(&self, who: Vec<impl Into<MultiAddressLike>>) -> SubmittableTransaction {
		let who: Vec<MultiAddressLike> = who.into_iter().map(|x| x.into()).collect();
		let who: Result<Vec<MultiAddress>, _> = who.into_iter().map(MultiAddress::try_from).collect();
		let who = who.expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::Kick { who };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Forces the commission for the given validator to the chain minimum.
	///
	/// # Panics
	/// Panics if `validator_stash` cannot be converted into an `AccountId`.
	///
	/// # Arguments
	/// * `validator_stash` - Stash account whose commission is being forced.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that enforces the minimum commission.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn force_apply_min_commission(&self, validator_stash: impl Into<AccountIdLike>) -> SubmittableTransaction {
		let validator_stash: AccountIdLike = validator_stash.into();
		let validator_stash = AccountId::try_from(validator_stash).expect("Malformed string is passed for AccountId");

		let value = avail::staking::tx::ForceApplyMinCommission { validator_stash };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Pays out staking rewards for a subset of nominators.
	///
	/// # Panics
	/// Panics if `validator_stash` cannot be converted into an `AccountId`.
	///
	/// # Arguments
	/// * `validator_stash` - Stash account whose rewards are being claimed.
	/// * `era` - Era for which rewards are paid.
	/// * `page` - Page index selecting which nominators to payout.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that triggers the paged payout.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
	/// # Arguments
	/// * `dest` - Destination account receiving the transfer.
	/// * `amount` - Amount to transfer.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that performs the transfer.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn transfer_allow_death(&self, dest: impl Into<MultiAddressLike>, amount: u128) -> SubmittableTransaction {
		let dest: MultiAddressLike = dest.into();
		let dest: MultiAddress = dest.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::balances::tx::TransferAllowDeath { dest, value: amount };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Transfers funds while keeping the sender's account alive.
	///
	/// # Panics
	/// Panics if `dest` cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `dest` - Destination account receiving the transfer.
	/// * `amount` - Amount to transfer.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that performs the keep-alive transfer.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn transfer_keep_alive(&self, dest: impl Into<MultiAddressLike>, amount: u128) -> SubmittableTransaction {
		let dest: MultiAddressLike = dest.into();
		let dest: MultiAddress = dest.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::balances::tx::TransferKeepAlive { dest, value: amount };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Transfers the entire free balance to the destination.
	///
	/// # Panics
	/// Panics if `dest` cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `dest` - Destination account receiving the transfer.
	/// * `keep_alive` - When `true`, leaves the minimum balance to keep the account alive.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that transfers the full balance.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn transfer_all(&self, dest: impl Into<MultiAddressLike>, keep_alive: bool) -> SubmittableTransaction {
		let dest: MultiAddressLike = dest.into();
		let dest: MultiAddress = dest.try_into().expect("Malformed string is passed for AccountId");

		let value = avail::balances::tx::TransferAll { dest, keep_alive };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
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
	/// # Arguments
	/// * `threshold` - Total number of approvals required to execute the call.
	/// * `other_signatories` - Remaining signatories excluding the caller.
	/// * `maybe_timepoint` - Optional timepoint identifying the in-progress multisig.
	/// * `call_hash` - Hash of the call being approved.
	/// * `max_weight` - Execution weight budget for the call.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that records the approval.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
			other_signatories.sort();

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

	/// Executes a multisig call with full call data.
	///
	/// # Panics
	/// Panics if any signatory identifier fails to convert into an `AccountId`.
	///
	/// # Arguments
	/// * `threshold` - Total number of approvals required to execute the call.
	/// * `other_signatories` - Remaining signatories excluding the caller.
	/// * `maybe_timepoint` - Optional timepoint identifying the in-progress multisig.
	/// * `call` - Call payload to execute once approvals are satisfied.
	/// * `max_weight` - Execution weight budget for the call.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that submits the multisig call.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
			other_signatories.sort();

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

	/// Executes a multisig call with a threshold of one.
	///
	/// # Panics
	/// Panics if any signatory identifier fails to convert into an `AccountId`.
	///
	/// # Arguments
	/// * `other_signatories` - Remaining signatories excluding the caller; used to derive the multisig account.
	/// * `call` - Call payload to execute.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that executes the call with a threshold of one.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn as_multi_threshold_1(
		&self,
		other_signatories: Vec<impl Into<AccountIdLike>>,
		call: impl Into<ExtrinsicCall>,
	) -> SubmittableTransaction {
		fn inner(client: Client, other_signatories: Vec<AccountIdLike>, call: ExtrinsicCall) -> SubmittableTransaction {
			let other_signatories: Result<Vec<AccountId>, _> =
				other_signatories.into_iter().map(|x| x.try_into()).collect();
			let mut other_signatories = other_signatories.expect("Malformed string is passed for AccountId");
			other_signatories.sort();

			let value = avail::multisig::tx::AsMultiThreshold1 { other_signatories, call };
			SubmittableTransaction::from_encodable(client, value)
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
	/// # Arguments
	/// * `threshold` - Total number of approvals required by the multisig.
	/// * `other_signatories` - Remaining signatories excluding the caller.
	/// * `timepoint` - Timepoint returned when the call was created.
	/// * `call_hash` - Hash of the call being cancelled.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that cancels the multisig operation.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
			other_signatories.sort();

			let call_hash: H256 = call_hash.try_into().expect("Malformed string is passed for H256");

			let value = avail::multisig::tx::CancelAsMulti { threshold, other_signatories, timepoint, call_hash };
			SubmittableTransaction::from_encodable(client, value)
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
	/// # Arguments
	/// * `key` - Application key bytes or string accepted by the runtime.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that registers the application key.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn create_application_key<'a>(&self, key: impl Into<StringOrBytes<'a>>) -> SubmittableTransaction {
		let key: Vec<u8> = Into::<StringOrBytes>::into(key).into();
		let value = avail::data_availability::tx::CreateApplicationKey { key };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Submits application data for availability guarantees.
	///
	/// # Arguments
	/// * `data` - Data payload to submit.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that submits the data for availability.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn submit_data<'a>(&self, app_id: u32, data: impl Into<StringOrBytes<'a>>) -> SubmittableTransaction {
		let data: Vec<u8> = Into::<StringOrBytes>::into(data).into();
		let value = avail::data_availability::tx::SubmitData { app_id, data };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Submits metadata describing an out-of-band blob.
	///
	/// # Arguments
	/// * `blob_hash` - Hash identifying the blob payload.
	/// * `size` - Size of the blob in bytes.
	/// * `commitments` - Commitment bytes used for verification.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] ready to be signed and submitted.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn submit_blob_metadata(
		&self,
		app_id: u32,
		blob_hash: H256,
		size: u64,
		commitments: Vec<u8>,
		eval_point_seed: Option<[u8; 32]>,
		eval_claim: Option<[u8; 16]>,
	) -> SubmittableTransaction {
		let value = avail::data_availability::tx::SubmitBlobMetadata { app_id, blob_hash, size, commitments, eval_point_seed, eval_claim };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}

/// Builds extrinsics for the `utility` pallet.
pub struct Utility(Client);
impl Utility {
	/// Dispatches a set of calls sequentially, aborting on failure.
	///
	/// # Arguments
	/// * `calls` - Calls executed in sequence.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that batches the supplied calls.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn batch(&self, calls: Vec<impl Into<ExtrinsicCall>>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::Batch::new();
		batch.add_calls(calls.into_iter().map(|x| x.into()).collect());
		SubmittableTransaction::from_encodable(self.0.clone(), batch)
	}

	/// Dispatches a set of calls and reverts the whole batch if any fail.
	///
	/// # Arguments
	/// * `calls` - Calls executed atomically; any failure rolls back the batch.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that executes the all-or-nothing batch.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn batch_all(&self, calls: Vec<impl Into<ExtrinsicCall>>) -> SubmittableTransaction {
		let mut batch = avail::utility::tx::BatchAll::new();
		batch.add_calls(calls.into_iter().map(|x| x.into()).collect());
		SubmittableTransaction::from_encodable(self.0.clone(), batch)
	}

	/// Dispatches a set of calls while ignoring failures.
	///
	/// # Arguments
	/// * `calls` - Calls executed sequentially; individual failures are ignored.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that executes the tolerant batch.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
	/// # Arguments
	/// * `id` - Proxy account that will dispatch the call.
	/// * `force_proxy_type` - Optional proxy type override.
	/// * `call` - Call to execute through the proxy.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that executes the proxied call.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

	/// Registers a new proxy delegate for the caller.
	///
	/// # Panics
	/// Panics if `id` cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `id` - Delegate account that gains proxy rights.
	/// * `proxy_type` - Proxy type applied to the delegate.
	/// * `delay` - Number of blocks the proxy must wait before first use.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that adds the proxy.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

	/// Removes a specific proxy delegate.
	///
	/// # Panics
	/// Panics if `delegate` cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `delegate` - Delegate being removed.
	/// * `proxy_type` - Proxy type to revoke.
	/// * `delay` - Expected delay recorded for the delegate.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that removes the proxy.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

	/// Removes all proxies belonging to the caller.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that clears the caller's proxies.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn remove_proxies(&self) -> SubmittableTransaction {
		let value = avail::proxy::tx::RemoveProxies {};
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Creates a pure proxy account with the requested parameters.
	///
	/// # Arguments
	/// * `proxy_type` - Proxy type to associate with the pure proxy.
	/// * `delay` - Number of blocks the proxy must wait before use.
	/// * `index` - Index differentiating multiple pure proxies.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that spawns the pure proxy.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn create_pure(&self, proxy_type: ProxyType, delay: u32, index: u16) -> SubmittableTransaction {
		let value = avail::proxy::tx::CreatePure { proxy_type, delay, index };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Kills a pure proxy that was previously spawned by the provided account.
	///
	/// # Panics
	/// Panics if `spawner` cannot be converted into a `MultiAddress`.
	///
	/// # Arguments
	/// * `spawner` - Account that originally spawned the pure proxy.
	/// * `proxy_type` - Proxy type associated with the pure proxy.
	/// * `index` - Index of the pure proxy to kill.
	/// * `height` - Block height recorded at spawn time.
	/// * `ext_index` - Extrinsic index recorded at spawn time.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that destroys the pure proxy.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

/// Builds extrinsics for the `vector` pallet.
///
/// Several helpers convert hash-like parameters into `H256` values and will panic if the provided
/// data cannot be parsed.
pub struct Vector(Client);
impl Vector {
	/// Submits a fulfillment proof for a pending cross-chain call.
	///
	/// # Arguments
	/// * `function_id` - Identifier of the function being fulfilled.
	/// * `input` - Encoded input payload.
	/// * `output` - Encoded output payload.
	/// * `proof` - Proof bytes attesting to the fulfillment.
	/// * `slot` - Slot in which the message was queued.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that fulfills the cross-chain call.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
	/// # Arguments
	/// * `slot` - Slot to execute.
	/// * `addr_message` - Addressed message payload.
	/// * `account_proof` - Proof for the account tree.
	/// * `storage_proof` - Proof for the storage entries.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that executes the message.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
	/// # Arguments
	/// * `source_chain_id` - Identifier of the source chain.
	/// * `frozen` - Boolean indicating the desired freeze state.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that updates the frozen state.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn source_chain_froze(&self, source_chain_id: u32, frozen: bool) -> SubmittableTransaction {
		let value = avail::vector::tx::SourceChainFroze { source_chain_id, frozen };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Sends a vector message to the specified domain.
	///
	/// # Panics
	/// Panics if `to` cannot be converted into an `H256`.
	///
	/// # Arguments
	/// * `message` - Message payload to send.
	/// * `to` - Destination address encoded as a hash string.
	/// * `domain` - Destination domain identifier.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that enqueues the message.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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

	/// Marks previous outbound messages as failed by index.
	///
	/// # Arguments
	/// * `failed_txs` - Indices of failed outbound messages.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that records the failure.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn failed_send_message_txs(&self, failed_txs: Vec<u32>) -> SubmittableTransaction {
		let value = avail::vector::tx::FailedSendMessageTxs { failed_txs };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the Poseidon hash commitment for a sync period.
	///
	/// # Arguments
	/// * `period` - Period identifier.
	/// * `poseidon_hash` - Poseidon hash commitment bytes.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that stores the commitment.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_poseidon_hash(&self, period: u64, poseidon_hash: Vec<u8>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetPoseidonHash { period: period.into(), poseidon_hash };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Registers the broadcaster for a specific domain.
	///
	/// # Arguments
	/// * `broadcaster_domain` - Domain where the broadcaster operates.
	/// * `broadcaster` - Broadcaster identifier.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that sets the broadcaster.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_broadcaster(&self, broadcaster_domain: u32, broadcaster: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetBroadcaster { broadcaster_domain: broadcaster_domain.into(), broadcaster };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Overwrites the set of domains allowed to send messages.
	///
	/// # Arguments
	/// * `value` - Domains permitted to send messages.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that updates the whitelist.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_whitelisted_domains(&self, value: Vec<u32>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetWhitelistedDomains { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the vector configuration parameters.
	///
	/// # Arguments
	/// * `value` - Configuration structure applied to the pallet.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that stores the configuration.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_configuration(&self, value: avail::vector::types::Configuration) -> SubmittableTransaction {
		let value = avail::vector::tx::SetConfiguration { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the function identifiers used by the pallet.
	///
	/// # Arguments
	/// * `value` - Optional tuple containing new function identifiers.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that records the identifiers.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_function_ids(&self, value: Option<(H256, H256)>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetFunctionIds { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Sets the verification key for the step circuit.
	///
	/// # Arguments
	/// * `value` - Optional verification key bytes.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that updates the verification key.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_step_verification_key(&self, value: Option<Vec<u8>>) -> SubmittableTransaction {
		let value = avail::vector::tx::SetStepVerificationKey { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the updater account hash.
	///
	/// # Arguments
	/// * `updater` - New updater hash.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that stores the updater hash.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_updater(&self, updater: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetUpdater { updater };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Submits a zero-knowledge proof fulfilling a pending message.
	///
	/// # Arguments
	/// * `proof` - Proof bytes attesting to message fulfillment.
	/// * `public_values` - Public inputs used during verification.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that fulfills the message.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn fulfill(&self, proof: Vec<u8>, public_values: Vec<u8>) -> SubmittableTransaction {
		let value = avail::vector::tx::Fulfill { proof, public_values };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Sets the verification key for SP1 proofs.
	///
	/// # Arguments
	/// * `sp1_vk` - SP1 verification key hash.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that stores the key.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_sp1_verification_key(&self, sp1_vk: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetSp1VerificationKey { sp1_vk };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Updates the sync committee hash for the provided period.
	///
	/// # Arguments
	/// * `period` - Period identifier.
	/// * `hash` - New sync committee hash.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that stores the sync committee hash.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_sync_committee_hash(&self, period: u64, hash: H256) -> SubmittableTransaction {
		let value = avail::vector::tx::SetSyncCommitteeHash { period, hash };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Enables or disables mock execution mode.
	///
	/// # Arguments
	/// * `value` - `true` to enable mock mode, `false` to disable.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that toggles mock mode.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn enable_mock(&self, value: bool) -> SubmittableTransaction {
		let value = avail::vector::tx::EnableMock { value };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Fulfills a message when running in mock mode.
	///
	/// # Arguments
	/// * `public_values` - Mock public values consumed by the fulfillment.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that fulfills the message in mock mode.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
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
	/// # Arguments
	/// * `remark` - Payload recorded on chain.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that emits the remark.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn remark(&self, remark: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::Remark { remark };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Replaces the runtime code with a new version.
	///
	/// # Arguments
	/// * `code` - WASM runtime bytecode.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that schedules the code upgrade.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_code(&self, code: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::SetCode { code };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Replaces the runtime code without performing standard checks.
	///
	/// # Arguments
	/// * `code` - WASM runtime bytecode.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that forces the code upgrade.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn set_code_without_checks(&self, code: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::SetCodeWithoutChecks { code };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}

	/// Emits a remark while guaranteeing an event is produced.
	///
	/// # Arguments
	/// * `remark` - Payload recorded on chain.
	///
	/// # Returns
	/// Returns a [`SubmittableTransaction`] that emits the remark alongside an event.
	///
	/// # Errors
	/// Does not perform network calls; transaction construction never fails.
	pub fn remark_with_event(&self, remark: Vec<u8>) -> SubmittableTransaction {
		let value = avail::system::tx::RemarkWithEvent { remark };
		SubmittableTransaction::from_encodable(self.0.clone(), value)
	}
}
