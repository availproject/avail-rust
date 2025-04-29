use crate::{
	api_dev::api::runtime_types::{pallet_staking::ValidatorPrefs, sp_arithmetic::per_things::Perbill},
	avail, AccountId, Client, SubmittableTransaction,
};
use subxt_core::utils::MultiAddress;

pub type BondCall = avail::staking::calls::types::Bond;
pub type BondExtraCall = avail::staking::calls::types::BondExtra;
pub type ChillCall = avail::staking::calls::types::Chill;
pub type ChillOtherCall = avail::staking::calls::types::ChillOther;
pub type NominateCall = avail::staking::calls::types::Nominate;
pub type UnbondCall = avail::staking::calls::types::Unbond;
pub type ValidateCall = avail::staking::calls::types::Validate;
pub type PayoutStakersCall = avail::staking::calls::types::PayoutStakers;
pub type RewardDestination = avail::runtime_types::pallet_staking::RewardDestination<AccountId>;

#[derive(Clone)]
pub struct Staking {
	pub client: Client,
}

pub struct Commission(u8);
impl Commission {
	pub fn new(value: u8) -> Result<Self, String> {
		if value > 100 {
			return Err(String::from("Commission cannot be more than 100"));
		}

		Ok(Self(value))
	}
}

impl Staking {
	/// Take the origin account as a stash and lock up `value` of its balance. `controller` will
	/// be the account that controls it.
	pub fn bond(&self, value: u128, payee: RewardDestination) -> SubmittableTransaction<BondCall> {
		let payload = avail::tx().staking().bond(value, payee);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Add some extra amount that have appeared in the stash `free_balance` into the balance up
	/// for staking.
	pub fn bond_extra(&self, max_additional: u128) -> SubmittableTransaction<BondExtraCall> {
		let payload = avail::tx().staking().bond_extra(max_additional);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Declare no desire to either validate or nominate.
	///
	/// Effects will be felt at the beginning of the next era.
	pub fn chill(&self) -> SubmittableTransaction<ChillCall> {
		let payload = avail::tx().staking().chill();
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Declare a `controller` to stop participating as either a validator or nominator.
	///
	/// Effects will be felt at the beginning of the next era.
	///
	/// The dispatch origin for this call must be _Signed_, but can be called by anyone.
	///
	/// If the caller is the same as the controller being targeted, then no further checks are
	/// enforced, and this function behaves just like `chill`.
	///
	/// If the caller is different than the controller being targeted, the following conditions
	/// must be met:
	///
	/// * `controller` must belong to a nominator who has become non-decodable,
	///
	/// Or:
	///
	///   - A `ChillThreshold` must be set and checked which defines how close to the max
	///     nominators or validators we must reach before users can start chilling one-another.
	///   - A `MaxNominatorCount` and `MaxValidatorCount` must be set which is used to determine
	///     how close we are to the threshold.
	///   - A `MinNominatorBond` and `MinValidatorBond` must be set and checked, which determines
	///     if this is a person that should be chilled because they have not met the threshold
	///     bond required.
	///
	/// This can be helpful if bond requirements are updated, and we need to remove old users
	/// who do not satisfy these requirements.
	pub fn chill_other(&self, stash: AccountId) -> SubmittableTransaction<ChillOtherCall> {
		let payload = avail::tx().staking().chill_other(stash);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Declare the desire to nominate `targets` for the origin controller.
	///
	/// Effects will be felt at the beginning of the next era.
	pub fn nominate(&self, targets: &[AccountId]) -> SubmittableTransaction<NominateCall> {
		let targets = targets.iter().map(|a| MultiAddress::Id(a.clone())).collect();

		let payload = avail::tx().staking().nominate(targets);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Schedule a portion of the stash to be unlocked ready for transfer out after the bond
	/// period ends. If this leaves an amount actively bonded less than
	/// T::Currency::minimum_balance(), then it is increased to the full amount.
	pub fn unbond(&self, value: u128) -> SubmittableTransaction<UnbondCall> {
		let payload = avail::tx().staking().unbond(value);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Declare the desire to validate for the origin controller.
	///
	/// Effects will be felt at the beginning of the next era.
	pub fn validate(&self, commission: Commission, blocked: bool) -> SubmittableTransaction<ValidateCall> {
		let commission = Perbill(commission.0 as u32);
		let perfs = ValidatorPrefs { commission, blocked };

		let payload = avail::tx().staking().validate(perfs);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	/// Pay out next page of the stakers behind a validator for the given era.
	///
	/// - `validator_stash` is the stash account of the validator.
	/// - `era` may be any era between `[current_era - history_depth; current_era]`.
	pub fn payout_stakers(&self, validator_stash: AccountId, era: u32) -> SubmittableTransaction<PayoutStakersCall> {
		let payload = avail::tx().staking().payout_stakers(validator_stash, era);
		SubmittableTransaction::new(self.client.clone(), payload)
	}
}
