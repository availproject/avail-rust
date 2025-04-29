pub use crate::avail::{
	nomination_pools::calls::types::{set_claim_permission::Permission, set_state::State},
	runtime_types::pallet_nomination_pools::BondExtra,
};
use crate::{
	avail::{
		self, nomination_pools::calls::types::set_commission::NewCommission as NewCommissionOriginal,
		runtime_types::sp_arithmetic::per_things::Perbill,
	},
	AccountId, Client, SubmittableTransaction,
};

pub type NominateCall = avail::nomination_pools::calls::types::Nominate;
pub type JoinCall = avail::nomination_pools::calls::types::Join;
pub type CreateCall = avail::nomination_pools::calls::types::Create;
pub type CreateWithPoolIdCall = avail::nomination_pools::calls::types::CreateWithPoolId;
pub type BondExtraCall = avail::nomination_pools::calls::types::BondExtra;
pub type BondExtraOtherCall = avail::nomination_pools::calls::types::BondExtraOther;
pub type SetCommissionCall = avail::nomination_pools::calls::types::SetCommission;
pub type SetClaimPermissionCall = avail::nomination_pools::calls::types::SetClaimPermission;
pub type SetStateCall = avail::nomination_pools::calls::types::SetState;
pub type ClaimPayoutCall = avail::nomination_pools::calls::types::ClaimPayout;
pub type ClaimPayoutOtherCall = avail::nomination_pools::calls::types::ClaimPayoutOther;
pub type ChillCall = avail::nomination_pools::calls::types::Chill;
pub type ClaimCommissionCall = avail::nomination_pools::calls::types::ClaimCommission;
pub type UnbondCall = avail::nomination_pools::calls::types::Unbond;
pub type SetMetadataCall = avail::nomination_pools::calls::types::SetMetadata;
pub type WithdrawUnbondedCall = avail::nomination_pools::calls::types::WithdrawUnbonded;

#[derive(Debug, Clone)]
pub struct NewCommission {
	pub amount: Perbill,
	pub payee: AccountId,
}

#[derive(Clone)]
pub struct NominationPools {
	pub client: Client,
}

impl NominationPools {
	/// Nominate on behalf of the pool.
	///
	/// The dispatch origin of this call must be signed by the pool nominator or the pool
	/// root role.
	///
	/// This directly forward the call to the staking pallet, on behalf of the pool bonded
	/// account.
	pub fn nominate(&self, pool_id: u32, validators: Vec<AccountId>) -> SubmittableTransaction<NominateCall> {
		let payload = avail::tx().nomination_pools().nominate(pool_id, validators);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Stake funds with a pool. The amount to bond is transferred from the member to the
	/// pools account and immediately increases the pools bond.
	///
	/// # Note
	///
	///   - An account can only be a member of a single pool.
	///   - An account cannot join the same pool multiple times.
	///   - This call will *not* dust the member account, so the member must have at least
	///     `existential deposit + amount` in their account.
	///   - Only a pool with [`PoolState::Open`] can be joined
	pub fn join(&self, amount: u128, pool_id: u32) -> SubmittableTransaction<JoinCall> {
		let payload = avail::tx().nomination_pools().join(amount, pool_id);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Create a new delegation pool with a previously used pool id
	///
	/// # Arguments
	///
	/// same as `create` with the inclusion of
	/// * `pool_id` - `A valid PoolId.
	pub fn create_with_pool_id(
		&self,
		amount: u128,
		root: AccountId,
		nominator: AccountId,
		bouncer: AccountId,
		pool_id: u32,
	) -> SubmittableTransaction<CreateWithPoolIdCall> {
		let payload = avail::tx().nomination_pools().create_with_pool_id(
			amount,
			root.into(),
			nominator.into(),
			bouncer.into(),
			pool_id,
		);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Create a new delegation pool.
	///
	/// # Arguments
	///
	///   - `amount` - The amount of funds to delegate to the pool. This also acts of a sort of
	///     deposit since the pools creator cannot fully unbond funds until the pool is being
	///     destroyed.
	///   - `index` - A disambiguation index for creating the account. Likely only useful when
	///     creating multiple pools in the same extrinsic.
	///   - `root` - The account to set as [`PoolRoles::root`].
	///   - `nominator` - The account to set as the [`PoolRoles::nominator`].
	///   - `bouncer` - The account to set as the [`PoolRoles::bouncer`].
	///
	/// # Note
	///
	/// In addition to `amount`, the caller will transfer the existential deposit; so the caller
	/// needs at have at least `amount + existential_deposit` transferable.
	pub fn create(
		&self,
		amount: u128,
		root: AccountId,
		nominator: AccountId,
		bouncer: AccountId,
	) -> SubmittableTransaction<CreateCall> {
		let payload = avail::tx()
			.nomination_pools()
			.create(amount, root.into(), nominator.into(), bouncer.into());
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Bond `extra` more funds from `origin` into the pool to which they already belong.
	///
	/// Additional funds can come from either the free balance of the account, of from the
	/// accumulated rewards, see [`BondExtra`].
	///
	/// Bonding extra funds implies an automatic payout of all pending rewards as well.
	/// See `bond_extra_other` to bond pending rewards of `other` members.
	pub fn bond_extra(&self, extra: BondExtra<u128>) -> SubmittableTransaction<BondExtraCall> {
		let payload = avail::tx().nomination_pools().bond_extra(extra);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Set the commission of a pool.
	///
	/// Both a commission percentage and a commission payee must be provided in the `current`
	/// tuple. Where a `current` of `None` is provided, any current commission will be removed.
	///
	/// - If a `None` is supplied to `new_commission`, existing commission will be removed.
	pub fn set_commission(
		&self,
		pool_id: u32,
		new_commission: Option<NewCommission>,
	) -> SubmittableTransaction<SetCommissionCall> {
		let new_commission: NewCommissionOriginal = match new_commission {
			Some(x) => Some((x.amount, x.payee)),
			None => None,
		};

		let payload = avail::tx().nomination_pools().set_commission(pool_id, new_commission);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Set a new state for the pool.
	///
	/// If a pool is already in the `Destroying` state, then under no condition can its state
	/// change again.
	///
	/// The dispatch origin of this call must be either:
	///
	///  1. signed by the bouncer, or the root role of the pool,
	///  2. if the pool conditions to be open are NOT met (as described by `ok_to_be_open`), and
	///     then the state of the pool can be permissionlessly changed to `Destroying`.
	pub fn set_state(&self, pool_id: u32, state: State) -> SubmittableTransaction<SetStateCall> {
		let payload = avail::tx().nomination_pools().set_state(pool_id, state);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// A bonded member can use this to claim their payout based on the rewards that the pool
	/// has accumulated since their last claimed payout (OR since joining if this is their first
	/// time claiming rewards). The payout will be transferred to the member's account.
	///
	/// The member will earn rewards pro rata based on the members stake vs the sum of the
	/// members in the pools stake. Rewards do not "expire".
	///
	/// See `claim_payout_other` to caim rewards on bahalf of some `other` pool member.
	pub fn claim_payout(&self) -> SubmittableTransaction<ClaimPayoutCall> {
		let payload = avail::tx().nomination_pools().claim_payout();
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Chill on behalf of the pool.
	///
	/// The dispatch origin of this call must be signed by the pool nominator or the pool
	/// root role, same as [`Pallet::nominate`].
	///
	/// This directly forward the call to the staking pallet, on behalf of the pool bonded
	/// account.
	pub fn chill(&self, pool_id: u32) -> SubmittableTransaction<ChillCall> {
		let payload = avail::tx().nomination_pools().chill(pool_id);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Allows a pool member to set a claim permission to allow or disallow permissionless
	/// bonding and withdrawing.
	///
	/// By default, this is `Permissioned`, which implies only the pool member themselves can
	/// claim their pending rewards. If a pool member wishes so, they can set this to
	/// `PermissionlessAll` to allow any account to claim their rewards and bond extra to the
	/// pool.
	///
	/// # Arguments
	///
	/// * `origin` - Member of a pool.
	/// * `actor` - Account to claim reward. // improve this
	pub fn set_claim_permission(&self, permission: Permission) -> SubmittableTransaction<SetClaimPermissionCall> {
		let payload = avail::tx().nomination_pools().set_claim_permission(permission);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Claim pending commission.
	///
	/// The dispatch origin of this call must be signed by the `root` role of the pool. Pending
	/// commission is paid out and added to total claimed commission`. Total pending commission
	/// is reset to zero. the current.
	pub fn claim_commission(&self, pool_id: u32) -> SubmittableTransaction<ClaimCommissionCall> {
		let payload = avail::tx().nomination_pools().claim_commission(pool_id);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// `origin` can claim payouts on some pool member `other`'s behalf.
	///
	/// Pool member `other` must have a `PermissionlessAll` or `PermissionlessWithdraw` in order
	/// for this call to be successful.
	pub fn claim_payout_other(&self, other: AccountId) -> SubmittableTransaction<ClaimPayoutOtherCall> {
		let payload = avail::tx().nomination_pools().claim_payout_other(other);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Unbond up to `unbonding_points` of the `member_account`'s funds from the pool. It
	/// implicitly collects the rewards one last time, since not doing so would mean some
	/// rewards would be forfeited.
	///
	/// Under certain conditions, this call can be dispatched permissionlessly (i.e. by any
	/// account).
	///
	/// # Conditions for a permissionless dispatch.
	///
	///   - The pool is blocked and the caller is either the root or bouncer. This is refereed to
	///     as a kick.
	///   - The pool is destroying and the member is not the depositor.
	///   - The pool is destroying, the member is the depositor and no other members are in the
	///     pool.
	///
	/// ## Conditions for permissioned dispatch (i.e. the caller is also the
	/// `member_account`):
	///
	///   - The caller is not the depositor.
	///   - The caller is the depositor, the pool is destroying and no other members are in the
	///     pool.
	///
	/// # Note
	///
	/// If there are too many unlocking chunks to unbond with the pool account,
	/// [`Call::pool_withdraw_unbonded`] can be called to try and minimize unlocking chunks.
	/// The [`StakingInterface::unbond`] will implicitly call [`Call::pool_withdraw_unbonded`]
	/// to try to free chunks if necessary (ie. if unbound was called and no unlocking chunks
	/// are available). However, it may not be possible to release the current unlocking chunks,
	/// in which case, the result of this call will likely be the `NoMoreChunks` error from the
	/// staking system.
	pub fn unbond(&self, member_account: AccountId, unbonding_points: u128) -> SubmittableTransaction<UnbondCall> {
		let payload = avail::tx()
			.nomination_pools()
			.unbond(member_account.into(), unbonding_points);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Set a new metadata for the pool.
	///
	/// The dispatch origin of this call must be signed by the bouncer, or the root role of the
	/// pool.
	pub fn set_metadata(&self, pool_id: u32, metadata: Vec<u8>) -> SubmittableTransaction<SetMetadataCall> {
		let payload = avail::tx().nomination_pools().set_metadata(pool_id, metadata);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Withdraw unbonded funds from `member_account`. If no bonded funds can be unbonded, an
	/// error is returned.
	///
	/// Under certain conditions, this call can be dispatched permissionlessly (i.e. by any
	/// account).
	///
	/// # Conditions for a permissionless dispatch
	///
	/// * The pool is in destroy mode and the target is not the depositor.
	/// * The target is the depositor and they are the only member in the sub pools.
	/// * The pool is blocked and the caller is either the root or bouncer.
	///
	/// # Conditions for permissioned dispatch
	///
	/// * The caller is the target and they are not the depositor.
	///
	/// # Note
	///
	/// If the target is the depositor, the pool will be destroyed.
	pub fn withdraw_unbonded(
		&self,
		member_account: AccountId,
		num_slashing_spans: u32,
	) -> SubmittableTransaction<WithdrawUnbondedCall> {
		let payload = avail::tx()
			.nomination_pools()
			.withdraw_unbonded(member_account.into(), num_slashing_spans);
		SubmittableTransaction::new(self.client.clone(), payload)
	}
}
