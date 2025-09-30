use super::{AccountId, MultiAddress};
use crate::{
	H256, HasHeader, StorageHasher, StorageMap, StorageValue, substrate::extrinsic::ExtrinsicCall,
	utils::decode_already_decoded,
};
use codec::{Compact, Decode, Encode};
use scale_decode::DecodeAsType;
use scale_encode::EncodeAsType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum RuntimeCall {
	BalancesTransferAllDeath(balances::tx::TransferAllowDeath),
	BalancesTransferKeepAlive(balances::tx::TransferKeepAlive),
	BalancesTransferAll(balances::tx::TransferAll),
	UtilityBatch(utility::tx::Batch),
	UtilityBatchAll(utility::tx::BatchAll),
	UtilityForceBatch(utility::tx::ForceBatch),
	SystemRemark(system::tx::Remark),
	SystemSetCode(system::tx::SetCode),
	SystemSetCodeWithoutChecks(system::tx::SetCodeWithoutChecks),
	SystemRemarkWithEvent(system::tx::RemarkWithEvent),
	ProxyProxy(proxy::tx::Proxy),
	ProxyAddProxy(proxy::tx::AddProxy),
	ProxyRemoveProxy(proxy::tx::RemoveProxy),
	ProxyRemoveProxies(proxy::tx::RemoveProxies),
	ProxyCreatePure(proxy::tx::CreatePure),
	ProxyKillPure(proxy::tx::KillPure),
	MultisigAsMultiThreshold1(multisig::tx::AsMultiThreshold1),
	MultisigAsMulti(multisig::tx::AsMulti),
	MultisigApproveAsMulti(multisig::tx::ApproveAsMulti),
	MultisigCancelAsMulti(multisig::tx::CancelAsMulti),
	DataAvailabilityCreateApplicationKey(data_availability::tx::CreateApplicationKey),
	DataAvailabilitySubmitData(data_availability::tx::SubmitData),
	StakingBond(staking::tx::Bond),
	StakingBondExtra(staking::tx::BondExtra),
	StakingChill(staking::tx::Chill),
	StakingChillOther(staking::tx::ChillOther),
	StakingForceApplyMinCommission(staking::tx::ForceApplyMinCommission),
	StakingKick(staking::tx::Kick),
	StakingNominate(staking::tx::Nominate),
	StakingPayoutStakers(staking::tx::PayoutStakers),
	StakingPayoutStakersByPage(staking::tx::PayoutStakersByPage),
	StakingReapStash(staking::tx::ReapStash),
	StakingRebond(staking::tx::Rebond),
	StakingSetController(staking::tx::SetController),
	StakingSetPayee(staking::tx::SetPayee),
	StakingUnbond(staking::tx::Unbond),
	StakingValidate(staking::tx::Validate),
	StakingWithdrawUnbonded(staking::tx::WithdrawUnbonded),
	NominationPoolsBondExtra(nomination_pools::tx::BondExtra),
	NominationPoolsBondExtraOther(nomination_pools::tx::BondExtraOther),
	NominationPoolsChill(nomination_pools::tx::Chill),
	NominationPoolsClaimCommission(nomination_pools::tx::ClaimCommission),
	NominationPoolsClaimPayout(nomination_pools::tx::ClaimPayout),
	NominationPoolsClaimPayoutOther(nomination_pools::tx::ClaimPayoutOther),
	NominationPoolsCreate(nomination_pools::tx::Create),
	NominationPoolsCreateWithPoolId(nomination_pools::tx::CreateWithPoolId),
	NominationPoolsJoin(nomination_pools::tx::Join),
	NominationPoolsNominate(nomination_pools::tx::Nominate),
	NominationPoolsSetClaimPermission(nomination_pools::tx::SetClaimPermission),
	NominationPoolsSetCommission(nomination_pools::tx::SetCommission),
	NominationPoolsSetCommissionChangeRate(nomination_pools::tx::SetCommissionChangeRate),
	NominationPoolsSetCommissionMax(nomination_pools::tx::SetCommissionMax),
	NominationPoolsSetMetadata(nomination_pools::tx::SetMetadata),
	NominationPoolsSetState(nomination_pools::tx::SetState),
	NominationPoolsUnbond(nomination_pools::tx::Unbond),
	NominationPoolsUpdateRoles(nomination_pools::tx::UpdateRoles),
	NominationPoolsWithdrawUnbonded(nomination_pools::tx::WithdrawUnbonded),
	SessionSetKeys(session::tx::SetKeys),
	SessionPurgeKeys(session::tx::PurgeKeys),
	TimestampSet(timestamp::tx::Set),
}
impl Encode for RuntimeCall {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		match self {
			RuntimeCall::BalancesTransferAllDeath(x) => x.encode_to(dest),
			RuntimeCall::BalancesTransferKeepAlive(x) => x.encode_to(dest),
			RuntimeCall::BalancesTransferAll(x) => x.encode_to(dest),
			RuntimeCall::UtilityBatch(x) => x.encode_to(dest),
			RuntimeCall::UtilityBatchAll(x) => x.encode_to(dest),
			RuntimeCall::UtilityForceBatch(x) => x.encode_to(dest),
			RuntimeCall::SystemRemark(x) => x.encode_to(dest),
			RuntimeCall::SystemSetCode(x) => x.encode_to(dest),
			RuntimeCall::SystemSetCodeWithoutChecks(x) => x.encode_to(dest),
			RuntimeCall::SystemRemarkWithEvent(x) => x.encode_to(dest),
			RuntimeCall::ProxyProxy(x) => x.encode_to(dest),
			RuntimeCall::ProxyAddProxy(x) => x.encode_to(dest),
			RuntimeCall::ProxyRemoveProxy(x) => x.encode_to(dest),
			RuntimeCall::ProxyRemoveProxies(x) => x.encode_to(dest),
			RuntimeCall::ProxyCreatePure(x) => x.encode_to(dest),
			RuntimeCall::ProxyKillPure(x) => x.encode_to(dest),
			RuntimeCall::MultisigAsMultiThreshold1(x) => x.encode_to(dest),
			RuntimeCall::MultisigAsMulti(x) => x.encode_to(dest),
			RuntimeCall::MultisigApproveAsMulti(x) => x.encode_to(dest),
			RuntimeCall::MultisigCancelAsMulti(x) => x.encode_to(dest),
			RuntimeCall::DataAvailabilityCreateApplicationKey(x) => x.encode_to(dest),
			RuntimeCall::DataAvailabilitySubmitData(x) => x.encode_to(dest),
			RuntimeCall::StakingBond(x) => x.encode_to(dest),
			RuntimeCall::StakingBondExtra(x) => x.encode_to(dest),
			RuntimeCall::StakingChill(x) => x.encode_to(dest),
			RuntimeCall::StakingChillOther(x) => x.encode_to(dest),
			RuntimeCall::StakingForceApplyMinCommission(x) => x.encode_to(dest),
			RuntimeCall::StakingKick(x) => x.encode_to(dest),
			RuntimeCall::StakingNominate(x) => x.encode_to(dest),
			RuntimeCall::StakingPayoutStakers(x) => x.encode_to(dest),
			RuntimeCall::StakingPayoutStakersByPage(x) => x.encode_to(dest),
			RuntimeCall::StakingReapStash(x) => x.encode_to(dest),
			RuntimeCall::StakingRebond(x) => x.encode_to(dest),
			RuntimeCall::StakingSetController(x) => x.encode_to(dest),
			RuntimeCall::StakingSetPayee(x) => x.encode_to(dest),
			RuntimeCall::StakingUnbond(x) => x.encode_to(dest),
			RuntimeCall::StakingValidate(x) => x.encode_to(dest),
			RuntimeCall::StakingWithdrawUnbonded(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsBondExtra(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsBondExtraOther(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsChill(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsClaimCommission(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsClaimPayout(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsClaimPayoutOther(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsCreate(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsCreateWithPoolId(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsJoin(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsNominate(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsSetClaimPermission(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsSetCommission(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsSetCommissionChangeRate(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsSetCommissionMax(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsSetMetadata(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsSetState(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsUnbond(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsUpdateRoles(x) => x.encode_to(dest),
			RuntimeCall::NominationPoolsWithdrawUnbonded(x) => x.encode_to(dest),
			RuntimeCall::SessionSetKeys(x) => x.encode_to(dest),
			RuntimeCall::SessionPurgeKeys(x) => x.encode_to(dest),
			RuntimeCall::TimestampSet(x) => x.encode_to(dest),
		}
	}
}
impl Decode for RuntimeCall {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let pallet_id = input.read_byte()?;
		let variant_id = input.read_byte()?;

		if pallet_id == timestamp::PALLET_ID {
			if variant_id == timestamp::tx::Set::HEADER_INDEX.1 {
				let call = timestamp::tx::Set::decode(input)?;
				return Ok(RuntimeCall::TimestampSet(call));
			}
		}

		if pallet_id == session::PALLET_ID {
			if variant_id == session::tx::SetKeys::HEADER_INDEX.1 {
				let call = session::tx::SetKeys::decode(input)?;
				return Ok(RuntimeCall::SessionSetKeys(call));
			}

			if variant_id == session::tx::PurgeKeys::HEADER_INDEX.1 {
				let call = session::tx::PurgeKeys::decode(input)?;
				return Ok(RuntimeCall::SessionPurgeKeys(call));
			}
		}

		if pallet_id == balances::PALLET_ID {
			if variant_id == balances::tx::TransferAllowDeath::HEADER_INDEX.1 {
				let call = balances::tx::TransferAllowDeath::decode(input)?;
				return Ok(RuntimeCall::BalancesTransferAllDeath(call));
			}

			if variant_id == balances::tx::TransferKeepAlive::HEADER_INDEX.1 {
				let call = balances::tx::TransferKeepAlive::decode(input)?;
				return Ok(RuntimeCall::BalancesTransferKeepAlive(call));
			}

			if variant_id == balances::tx::TransferAll::HEADER_INDEX.1 {
				let call = balances::tx::TransferAll::decode(input)?;
				return Ok(RuntimeCall::BalancesTransferAll(call));
			}
		}

		if pallet_id == utility::PALLET_ID {
			if variant_id == utility::tx::Batch::HEADER_INDEX.1 {
				let call = utility::tx::Batch::decode(input)?;
				return Ok(RuntimeCall::UtilityBatch(call));
			}

			if variant_id == utility::tx::BatchAll::HEADER_INDEX.1 {
				let call = utility::tx::BatchAll::decode(input)?;
				return Ok(RuntimeCall::UtilityBatchAll(call));
			}

			if variant_id == utility::tx::ForceBatch::HEADER_INDEX.1 {
				let call = utility::tx::ForceBatch::decode(input)?;
				return Ok(RuntimeCall::UtilityForceBatch(call));
			}
		}

		if pallet_id == system::PALLET_ID {
			if variant_id == system::tx::Remark::HEADER_INDEX.1 {
				let call = system::tx::Remark::decode(input)?;
				return Ok(RuntimeCall::SystemRemark(call));
			}

			if variant_id == system::tx::SetCode::HEADER_INDEX.1 {
				let call = system::tx::SetCode::decode(input)?;
				return Ok(RuntimeCall::SystemSetCode(call));
			}

			if variant_id == system::tx::SetCodeWithoutChecks::HEADER_INDEX.1 {
				let call = system::tx::SetCodeWithoutChecks::decode(input)?;
				return Ok(RuntimeCall::SystemSetCodeWithoutChecks(call));
			}

			if variant_id == system::tx::RemarkWithEvent::HEADER_INDEX.1 {
				let call = system::tx::RemarkWithEvent::decode(input)?;
				return Ok(RuntimeCall::SystemRemarkWithEvent(call));
			}
		}

		if pallet_id == proxy::PALLET_ID {
			if variant_id == proxy::tx::Proxy::HEADER_INDEX.1 {
				let call = proxy::tx::Proxy::decode(input)?;
				return Ok(RuntimeCall::ProxyProxy(call));
			}

			if variant_id == proxy::tx::AddProxy::HEADER_INDEX.1 {
				let call = proxy::tx::AddProxy::decode(input)?;
				return Ok(RuntimeCall::ProxyAddProxy(call));
			}

			if variant_id == proxy::tx::CreatePure::HEADER_INDEX.1 {
				let call = proxy::tx::CreatePure::decode(input)?;
				return Ok(RuntimeCall::ProxyCreatePure(call));
			}

			if variant_id == proxy::tx::KillPure::HEADER_INDEX.1 {
				let call = proxy::tx::KillPure::decode(input)?;
				return Ok(RuntimeCall::ProxyKillPure(call));
			}

			if variant_id == proxy::tx::RemoveProxies::HEADER_INDEX.1 {
				let call = proxy::tx::RemoveProxies::decode(input)?;
				return Ok(RuntimeCall::ProxyRemoveProxies(call));
			}

			if variant_id == proxy::tx::RemoveProxy::HEADER_INDEX.1 {
				let call = proxy::tx::RemoveProxy::decode(input)?;
				return Ok(RuntimeCall::ProxyRemoveProxy(call));
			}
		}

		if pallet_id == multisig::PALLET_ID {
			if variant_id == multisig::tx::ApproveAsMulti::HEADER_INDEX.1 {
				let call = multisig::tx::ApproveAsMulti::decode(input)?;
				return Ok(RuntimeCall::MultisigApproveAsMulti(call));
			}

			if variant_id == multisig::tx::AsMulti::HEADER_INDEX.1 {
				let call = multisig::tx::AsMulti::decode(input)?;
				return Ok(RuntimeCall::MultisigAsMulti(call));
			}

			if variant_id == multisig::tx::AsMultiThreshold1::HEADER_INDEX.1 {
				let call = multisig::tx::AsMultiThreshold1::decode(input)?;
				return Ok(RuntimeCall::MultisigAsMultiThreshold1(call));
			}

			if variant_id == multisig::tx::CancelAsMulti::HEADER_INDEX.1 {
				let call = multisig::tx::CancelAsMulti::decode(input)?;
				return Ok(RuntimeCall::MultisigCancelAsMulti(call));
			}
		}

		if pallet_id == data_availability::PALLET_ID {
			if variant_id == data_availability::tx::CreateApplicationKey::HEADER_INDEX.1 {
				let call = data_availability::tx::CreateApplicationKey::decode(input)?;
				return Ok(RuntimeCall::DataAvailabilityCreateApplicationKey(call));
			}

			if variant_id == data_availability::tx::SubmitData::HEADER_INDEX.1 {
				let call = data_availability::tx::SubmitData::decode(input)?;
				return Ok(RuntimeCall::DataAvailabilitySubmitData(call));
			}
		}

		if pallet_id == staking::PALLET_ID {
			if variant_id == staking::tx::Bond::HEADER_INDEX.1 {
				let call = staking::tx::Bond::decode(input)?;
				return Ok(RuntimeCall::StakingBond(call));
			}

			if variant_id == staking::tx::BondExtra::HEADER_INDEX.1 {
				let call = staking::tx::BondExtra::decode(input)?;
				return Ok(RuntimeCall::StakingBondExtra(call));
			}

			if variant_id == staking::tx::Chill::HEADER_INDEX.1 {
				let call = staking::tx::Chill::decode(input)?;
				return Ok(RuntimeCall::StakingChill(call));
			}

			if variant_id == staking::tx::ChillOther::HEADER_INDEX.1 {
				let call = staking::tx::ChillOther::decode(input)?;
				return Ok(RuntimeCall::StakingChillOther(call));
			}

			if variant_id == staking::tx::ForceApplyMinCommission::HEADER_INDEX.1 {
				let call = staking::tx::ForceApplyMinCommission::decode(input)?;
				return Ok(RuntimeCall::StakingForceApplyMinCommission(call));
			}

			if variant_id == staking::tx::Kick::HEADER_INDEX.1 {
				let call = staking::tx::Kick::decode(input)?;
				return Ok(RuntimeCall::StakingKick(call));
			}

			if variant_id == staking::tx::Nominate::HEADER_INDEX.1 {
				let call = staking::tx::Nominate::decode(input)?;
				return Ok(RuntimeCall::StakingNominate(call));
			}

			if variant_id == staking::tx::PayoutStakers::HEADER_INDEX.1 {
				let call = staking::tx::PayoutStakers::decode(input)?;
				return Ok(RuntimeCall::StakingPayoutStakers(call));
			}

			if variant_id == staking::tx::PayoutStakersByPage::HEADER_INDEX.1 {
				let call = staking::tx::PayoutStakersByPage::decode(input)?;
				return Ok(RuntimeCall::StakingPayoutStakersByPage(call));
			}

			if variant_id == staking::tx::ReapStash::HEADER_INDEX.1 {
				let call = staking::tx::ReapStash::decode(input)?;
				return Ok(RuntimeCall::StakingReapStash(call));
			}

			if variant_id == staking::tx::Rebond::HEADER_INDEX.1 {
				let call = staking::tx::Rebond::decode(input)?;
				return Ok(RuntimeCall::StakingRebond(call));
			}

			if variant_id == staking::tx::SetController::HEADER_INDEX.1 {
				let call = staking::tx::SetController::decode(input)?;
				return Ok(RuntimeCall::StakingSetController(call));
			}

			if variant_id == staking::tx::SetPayee::HEADER_INDEX.1 {
				let call = staking::tx::SetPayee::decode(input)?;
				return Ok(RuntimeCall::StakingSetPayee(call));
			}

			if variant_id == staking::tx::Unbond::HEADER_INDEX.1 {
				let call = staking::tx::Unbond::decode(input)?;
				return Ok(RuntimeCall::StakingUnbond(call));
			}

			if variant_id == staking::tx::Validate::HEADER_INDEX.1 {
				let call = staking::tx::Validate::decode(input)?;
				return Ok(RuntimeCall::StakingValidate(call));
			}

			if variant_id == staking::tx::WithdrawUnbonded::HEADER_INDEX.1 {
				let call = staking::tx::WithdrawUnbonded::decode(input)?;
				return Ok(RuntimeCall::StakingWithdrawUnbonded(call));
			}
		}

		if pallet_id == nomination_pools::PALLET_ID {
			if variant_id == nomination_pools::tx::BondExtra::HEADER_INDEX.1 {
				let call = nomination_pools::tx::BondExtra::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsBondExtra(call));
			}

			if variant_id == nomination_pools::tx::BondExtraOther::HEADER_INDEX.1 {
				let call = nomination_pools::tx::BondExtraOther::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsBondExtraOther(call));
			}

			if variant_id == nomination_pools::tx::Chill::HEADER_INDEX.1 {
				let call = nomination_pools::tx::Chill::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsChill(call));
			}

			if variant_id == nomination_pools::tx::ClaimCommission::HEADER_INDEX.1 {
				let call = nomination_pools::tx::ClaimCommission::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsClaimCommission(call));
			}

			if variant_id == nomination_pools::tx::ClaimPayout::HEADER_INDEX.1 {
				let call = nomination_pools::tx::ClaimPayout::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsClaimPayout(call));
			}

			if variant_id == nomination_pools::tx::ClaimPayoutOther::HEADER_INDEX.1 {
				let call = nomination_pools::tx::ClaimPayoutOther::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsClaimPayoutOther(call));
			}

			if variant_id == nomination_pools::tx::Create::HEADER_INDEX.1 {
				let call = nomination_pools::tx::Create::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsCreate(call));
			}

			if variant_id == nomination_pools::tx::CreateWithPoolId::HEADER_INDEX.1 {
				let call = nomination_pools::tx::CreateWithPoolId::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsCreateWithPoolId(call));
			}

			if variant_id == nomination_pools::tx::Join::HEADER_INDEX.1 {
				let call = nomination_pools::tx::Join::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsJoin(call));
			}

			if variant_id == nomination_pools::tx::Nominate::HEADER_INDEX.1 {
				let call = nomination_pools::tx::Nominate::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsNominate(call));
			}

			if variant_id == nomination_pools::tx::SetClaimPermission::HEADER_INDEX.1 {
				let call = nomination_pools::tx::SetClaimPermission::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsSetClaimPermission(call));
			}

			if variant_id == nomination_pools::tx::SetCommission::HEADER_INDEX.1 {
				let call = nomination_pools::tx::SetCommission::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsSetCommission(call));
			}

			if variant_id == nomination_pools::tx::SetCommissionChangeRate::HEADER_INDEX.1 {
				let call = nomination_pools::tx::SetCommissionChangeRate::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsSetCommissionChangeRate(call));
			}

			if variant_id == nomination_pools::tx::SetCommissionMax::HEADER_INDEX.1 {
				let call = nomination_pools::tx::SetCommissionMax::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsSetCommissionMax(call));
			}

			if variant_id == nomination_pools::tx::SetMetadata::HEADER_INDEX.1 {
				let call = nomination_pools::tx::SetMetadata::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsSetMetadata(call));
			}

			if variant_id == nomination_pools::tx::SetState::HEADER_INDEX.1 {
				let call = nomination_pools::tx::SetState::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsSetState(call));
			}

			if variant_id == nomination_pools::tx::Unbond::HEADER_INDEX.1 {
				let call = nomination_pools::tx::Unbond::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsUnbond(call));
			}

			if variant_id == nomination_pools::tx::UpdateRoles::HEADER_INDEX.1 {
				let call = nomination_pools::tx::UpdateRoles::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsUpdateRoles(call));
			}

			if variant_id == nomination_pools::tx::WithdrawUnbonded::HEADER_INDEX.1 {
				let call = nomination_pools::tx::WithdrawUnbonded::decode(input)?;
				return Ok(RuntimeCall::NominationPoolsWithdrawUnbonded(call));
			}
		}

		Err(codec::Error::from("Failed to decode runtime call"))
	}
}

pub mod data_availability {
	use super::*;
	pub const PALLET_ID: u8 = 29;

	pub mod storage {
		use super::{system::types::DispatchFeeModifier, *};

		pub struct NextAppId;
		impl StorageValue for NextAppId {
			type VALUE = Compact<u32>;

			const PALLET_NAME: &str = "DataAvailability";
			const STORAGE_NAME: &str = "NextAppId";
		}

		pub struct SubmitDataFeeModifier;
		impl StorageValue for SubmitDataFeeModifier {
			type VALUE = DispatchFeeModifier;

			const PALLET_NAME: &str = "DataAvailability";
			const STORAGE_NAME: &str = "SubmitDataFeeModifier";
		}

		pub struct AppKeys;
		impl StorageMap for AppKeys {
			type KEY = Vec<u8>;
			type VALUE = super::types::AppKey;

			const KEY_HASHER: StorageHasher = StorageHasher::Blake2_128Concat;
			const PALLET_NAME: &str = "DataAvailability";
			const STORAGE_NAME: &str = "AppKeys";
		}
	}

	pub mod types {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct AppKey {
			pub owner: AccountId,
			pub id: u32,
		}
		impl Encode for AppKey {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.owner.encode_to(dest);
				Compact::<u32>(self.id).encode_to(dest);
			}
		}
		impl Decode for AppKey {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let owner = Decode::decode(input)?;
				let id = Compact::<u32>::decode(input)?.0;
				Ok(Self { owner, id })
			}
		}
	}

	pub mod events {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct ApplicationKeyCreated {
			pub key: Vec<u8>,
			pub owner: AccountId,
			pub id: u32,
		}
		impl HasHeader for ApplicationKeyCreated {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}
		impl Encode for ApplicationKeyCreated {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.key.encode_to(dest);
				self.owner.encode_to(dest);
				Compact(self.id).encode_to(dest);
			}
		}
		impl Decode for ApplicationKeyCreated {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let key = Decode::decode(input)?;
				let owner = Decode::decode(input)?;
				let id = Compact::<u32>::decode(input)?.0;
				Ok(Self { key, owner, id })
			}
		}

		#[derive(Debug, Clone)]
		pub struct DataSubmitted {
			pub who: AccountId,
			pub data_hash: H256,
		}
		impl HasHeader for DataSubmitted {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
		impl Encode for DataSubmitted {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
				self.data_hash.encode_to(dest);
			}
		}
		impl Decode for DataSubmitted {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				let data_hash = Decode::decode(input)?;
				Ok(Self { who, data_hash })
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct CreateApplicationKey {
			pub key: Vec<u8>,
		}
		impl Encode for CreateApplicationKey {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.key.encode_to(dest);
			}
		}
		impl Decode for CreateApplicationKey {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let key = Decode::decode(input)?;
				Ok(Self { key })
			}
		}
		impl HasHeader for CreateApplicationKey {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct SubmitData {
			pub data: Vec<u8>,
		}
		impl Encode for SubmitData {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.data.encode_to(dest);
			}
		}
		impl Decode for SubmitData {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let data = Decode::decode(input)?;
				Ok(Self { data })
			}
		}
		impl HasHeader for SubmitData {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Clone)]
		pub struct SubmitBlobMetadata {
			pub blob_hash: H256,
			pub size: u64,
			pub commitments: Vec<u8>,
		}
		impl Encode for SubmitBlobMetadata {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.blob_hash.encode());
				dest.write(&self.size.encode());
				dest.write(&self.commitments.encode());
			}
		}
		impl Decode for SubmitBlobMetadata {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let blob_hash = Decode::decode(input)?;
				let size = Decode::decode(input)?;
				let commitments = Decode::decode(input)?;
				Ok(Self { blob_hash, size, commitments })
			}
		}
		impl HasHeader for SubmitBlobMetadata {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 5);
		}
	}
}

pub mod balances {
	use super::*;
	pub const PALLET_ID: u8 = 6;

	pub mod types {
		use super::*;

		#[derive(Debug, Default, Clone, DecodeAsType, EncodeAsType)]
		pub struct AccountData {
			pub free: u128,
			pub reserved: u128,
			pub frozen: u128,
			pub flags: u128,
		}

		impl Encode for AccountData {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.free.encode_to(dest);
				self.reserved.encode_to(dest);
				self.frozen.encode_to(dest);
				self.flags.encode_to(dest);
			}
		}
		impl Decode for AccountData {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let free = Decode::decode(input)?;
				let reserved = Decode::decode(input)?;
				let frozen = Decode::decode(input)?;
				let flags = Decode::decode(input)?;
				Ok(Self { free, reserved, frozen, flags })
			}
		}

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum BalanceStatus {
			/// Funds are free, as corresponding to `free` item in Balances.
			Free = 0,
			/// Funds are reserved, as corresponding to `reserved` item in Balances.
			Reserved = 1,
		}
		impl Encode for BalanceStatus {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
			}
		}
		impl Decode for BalanceStatus {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(BalanceStatus::Free),
					1 => Ok(BalanceStatus::Reserved),
					_ => Err("Failed to decode BalanceStatus Call. Unknown variant".into()),
				}
			}
		}
	}

	pub mod events {
		use super::*;

		/// An account was created with some free balance.
		#[derive(Debug, Clone)]
		pub struct Endowed {
			pub account: AccountId,
			pub free_balance: u128,
		}
		impl HasHeader for Endowed {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}
		impl Encode for Endowed {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.account.encode_to(dest);
				self.free_balance.encode_to(dest);
			}
		}
		impl Decode for Endowed {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let account = Decode::decode(input)?;
				let free_balance = Decode::decode(input)?;
				Ok(Self { account, free_balance })
			}
		}

		/// An account was removed whose balance was non-zero but below ExistentialDeposit,
		/// resulting in an outright loss.
		#[derive(Debug, Clone)]
		pub struct DustLost {
			pub account: AccountId,
			pub amount: u128,
		}
		impl HasHeader for DustLost {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
		impl Encode for DustLost {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.account.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for DustLost {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let account = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { account, amount })
			}
		}

		/// Transfer succeeded.
		#[derive(Debug, Clone)]
		pub struct Transfer {
			pub from: AccountId,
			pub to: AccountId,
			pub amount: u128,
		}
		impl HasHeader for Transfer {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}
		impl Encode for Transfer {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.from.encode_to(dest);
				self.to.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Transfer {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let from = Decode::decode(input)?;
				let to = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { from, to, amount })
			}
		}

		/// Some balance was reserved (moved from free to reserved).
		#[derive(Debug, Clone)]
		pub struct Reserved {
			pub who: AccountId,
			pub amount: u128,
		}
		impl HasHeader for Reserved {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
		impl Encode for Reserved {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Reserved {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { who, amount })
			}
		}

		/// Some balance was unreserved (moved from reserved to free).
		#[derive(Debug, Clone)]
		pub struct Unreserved {
			pub who: AccountId,
			pub amount: u128,
		}
		impl HasHeader for Unreserved {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 5);
		}
		impl Encode for Unreserved {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Unreserved {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { who, amount })
			}
		}

		/// Some amount was deposited (e.g. for transaction fees).
		#[derive(Debug, Clone)]
		pub struct Deposit {
			pub who: AccountId,
			pub amount: u128,
		}
		impl HasHeader for Deposit {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 7);
		}
		impl Encode for Deposit {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Deposit {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { who, amount })
			}
		}

		/// Some amount was withdrawn from the account (e.g. for transaction fees).
		#[derive(Debug, Clone)]
		pub struct Withdraw {
			pub who: AccountId,
			pub amount: u128,
		}
		impl HasHeader for Withdraw {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 8);
		}
		impl Encode for Withdraw {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Withdraw {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { who, amount })
			}
		}

		/// Some amount was removed from the account (e.g. for misbehavior).
		#[derive(Debug, Clone)]
		pub struct Slashed {
			pub who: AccountId,
			pub amount: u128,
		}
		impl HasHeader for Slashed {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 9);
		}
		impl Encode for Slashed {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Slashed {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { who, amount })
			}
		}

		/// Some balance was locked..
		#[derive(Debug, Clone)]
		pub struct Locked {
			pub who: AccountId,
			pub amount: u128,
		}
		impl HasHeader for Locked {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 17);
		}
		impl Encode for Locked {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Locked {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { who, amount })
			}
		}

		/// Some balance was unlocked.
		#[derive(Debug, Clone)]
		pub struct Unlocked {
			pub who: AccountId,
			pub amount: u128,
		}
		impl HasHeader for Unlocked {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 18);
		}
		impl Encode for Unlocked {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Unlocked {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { who, amount })
			}
		}

		/// Some balance was frozen.
		#[derive(Debug, Clone)]
		pub struct Frozen {
			pub who: AccountId,
			pub amount: u128,
		}
		impl HasHeader for Frozen {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 19);
		}
		impl Encode for Frozen {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Frozen {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { who, amount })
			}
		}

		/// Some balance was thawed.
		#[derive(Debug, Clone)]
		pub struct Thawed {
			pub who: AccountId,
			pub amount: u128,
		}
		impl HasHeader for Thawed {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 20);
		}
		impl Encode for Thawed {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Thawed {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { who, amount })
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct TransferAllowDeath {
			pub dest: MultiAddress,
			pub value: u128,
		}
		impl Encode for TransferAllowDeath {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.dest.encode_to(dest);
				Compact(self.value).encode_to(dest);
			}
		}
		impl Decode for TransferAllowDeath {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let dest = Decode::decode(input)?;
				let value = Compact::<u128>::decode(input)?.0;
				Ok(Self { dest, value })
			}
		}
		impl HasHeader for TransferAllowDeath {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct TransferKeepAlive {
			pub dest: MultiAddress,
			pub value: u128,
		}
		impl Encode for TransferKeepAlive {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.dest.encode_to(dest);
				Compact(self.value).encode_to(dest);
			}
		}
		impl Decode for TransferKeepAlive {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let dest = Decode::decode(input)?;
				let value = Compact::<u128>::decode(input)?.0;
				Ok(Self { dest, value })
			}
		}
		impl HasHeader for TransferKeepAlive {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Debug, Clone)]
		pub struct TransferAll {
			pub dest: MultiAddress,
			pub keep_alive: bool,
		}
		impl Encode for TransferAll {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.dest.encode_to(dest);
				self.keep_alive.encode_to(dest);
			}
		}
		impl Decode for TransferAll {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let dest = Decode::decode(input)?;
				let keep_alive = Decode::decode(input)?;
				Ok(Self { dest, keep_alive })
			}
		}
		impl HasHeader for TransferAll {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
	}
}

pub mod session {
	use super::*;
	pub const PALLET_ID: u8 = 11;

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct SetKeys {
			pub babe: H256,
			pub grandpa: H256,
			pub im_online: H256,
			pub authority_discovery: H256,
			pub proof: Vec<u8>,
		}
		impl Encode for SetKeys {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.babe.encode_to(dest);
				self.grandpa.encode_to(dest);
				self.im_online.encode_to(dest);
				self.authority_discovery.encode_to(dest);
				self.proof.encode_to(dest);
			}
		}
		impl Decode for SetKeys {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let babe = Decode::decode(input)?;
				let grandpa = Decode::decode(input)?;
				let im_online = Decode::decode(input)?;
				let authority_discovery = Decode::decode(input)?;
				let proof = Decode::decode(input)?;
				Ok(Self { babe, grandpa, im_online, authority_discovery, proof })
			}
		}
		impl HasHeader for SetKeys {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct PurgeKeys {}
		impl Encode for PurgeKeys {
			fn encode_to<T: codec::Output + ?Sized>(&self, _dest: &mut T) {}
		}
		impl Decode for PurgeKeys {
			fn decode<I: codec::Input>(_input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self {})
			}
		}
		impl HasHeader for PurgeKeys {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
	}
}

pub mod utility {
	use super::*;
	pub const PALLET_ID: u8 = 1;

	pub mod events {
		use super::*;

		/// Batch of dispatches did not complete fully. Index of first failing dispatch given, as
		/// well as the error.
		#[derive(Debug, Clone)]
		pub struct BatchInterrupted {
			pub index: u32,
			pub error: super::system::types::DispatchError,
		}
		impl HasHeader for BatchInterrupted {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}
		impl Decode for BatchInterrupted {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let index = Decode::decode(input)?;
				let error = Decode::decode(input)?;
				Ok(Self { index, error })
			}
		}

		/// Batch of dispatches completed fully with no error.
		#[derive(Debug, Clone)]
		pub struct BatchCompleted;
		impl HasHeader for BatchCompleted {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
		impl Decode for BatchCompleted {
			fn decode<I: codec::Input>(_input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self)
			}
		}

		/// Batch of dispatches completed but has error
		#[derive(Debug, Clone)]
		pub struct BatchCompletedWithErrors;
		impl HasHeader for BatchCompletedWithErrors {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}
		impl Decode for BatchCompletedWithErrors {
			fn decode<I: codec::Input>(_input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self)
			}
		}

		/// A single item within a Batch of dispatches has completed with no error
		#[derive(Debug, Clone)]
		pub struct ItemCompleted;
		impl HasHeader for ItemCompleted {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 3);
		}
		impl Decode for ItemCompleted {
			fn decode<I: codec::Input>(_input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self)
			}
		}

		/// A single item within a Batch of dispatches has completed with error.
		#[derive(Debug, Clone)]
		pub struct ItemFailed {
			pub error: super::system::types::DispatchError,
		}
		impl HasHeader for ItemFailed {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
		impl Decode for ItemFailed {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let error = Decode::decode(input)?;
				Ok(Self { error })
			}
		}

		/// A call was dispatched.
		#[derive(Debug, Clone)]
		pub struct DispatchedAs {
			pub result: Result<(), super::system::types::DispatchError>,
		}
		impl HasHeader for DispatchedAs {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 5);
		}
		impl Decode for DispatchedAs {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let result = Decode::decode(input)?;
				Ok(Self { result })
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Default, Clone)]
		pub struct Batch {
			length: u32,
			calls: Vec<u8>,
		}
		impl Batch {
			pub fn new() -> Self {
				Self::default()
			}

			pub fn decode_calls(&self) -> Result<Vec<RuntimeCall>, codec::Error> {
				if self.length == 0 {
					return Ok(Vec::new());
				}

				let mut runtime_calls: Vec<RuntimeCall> = Vec::with_capacity(self.length as usize);
				let mut calls = self.calls.as_slice();
				for _ in 0..self.length {
					runtime_calls.push(RuntimeCall::decode(&mut calls)?)
				}
				if !calls.is_empty() {
					return Err(codec::Error::from(
						"Bytes left in array. Failed to decode Batch call into RuntimeCalls",
					));
				}

				Ok(runtime_calls)
			}

			pub fn add_calls(&mut self, value: Vec<ExtrinsicCall>) {
				for v in value {
					self.add_call(v);
				}
			}

			pub fn add_call(&mut self, value: ExtrinsicCall) {
				self.length += 1;
				value.encode_to(&mut self.calls);
			}

			pub fn add_hex(&mut self, value: &str) -> Result<(), const_hex::FromHexError> {
				let decoded = const_hex::decode(value.trim_start_matches("0x"))?;
				self.add(decoded);
				Ok(())
			}

			pub fn add(&mut self, mut value: Vec<u8>) {
				self.length += 1;
				self.calls.append(&mut value);
			}

			pub fn len(&self) -> u32 {
				self.length
			}

			pub fn is_empty(&self) -> bool {
				self.len() == 0
			}

			pub fn calls(&self) -> &[u8] {
				&self.calls
			}
		}
		impl Encode for Batch {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.length).encode_to(dest);
				dest.write(&self.calls);
			}
		}
		impl Decode for Batch {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let length = Compact::<u32>::decode(input)?.0;
				let calls = decode_already_decoded(input)?;
				Ok(Self { length, calls })
			}
		}
		impl HasHeader for Batch {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Default, Clone)]
		pub struct BatchAll {
			length: u32,
			calls: Vec<u8>,
		}
		impl BatchAll {
			pub fn new() -> Self {
				Self::default()
			}

			pub fn decode_calls(&self) -> Result<Vec<RuntimeCall>, codec::Error> {
				if self.length == 0 {
					return Ok(Vec::new());
				}

				let mut runtime_calls: Vec<RuntimeCall> = Vec::with_capacity(self.length as usize);
				let mut calls = self.calls.as_slice();
				for _ in 0..self.length {
					runtime_calls.push(RuntimeCall::decode(&mut calls)?)
				}
				if !calls.is_empty() {
					return Err(codec::Error::from(
						"Bytes left in array. Failed to decode Batch call into RuntimeCalls",
					));
				}

				Ok(runtime_calls)
			}

			pub fn add_calls(&mut self, value: Vec<ExtrinsicCall>) {
				for v in value {
					self.add_call(v);
				}
			}

			pub fn add_call(&mut self, value: ExtrinsicCall) {
				self.length += 1;
				value.encode_to(&mut self.calls);
			}

			pub fn add_hex(&mut self, value: &str) -> Result<(), const_hex::FromHexError> {
				let decoded = const_hex::decode(value.trim_start_matches("0x"))?;
				self.add(decoded);
				Ok(())
			}

			pub fn add(&mut self, mut value: Vec<u8>) {
				self.length += 1;
				self.calls.append(&mut value);
			}

			pub fn len(&self) -> u32 {
				self.length
			}

			pub fn is_empty(&self) -> bool {
				self.len() == 0
			}

			pub fn calls(&self) -> &[u8] {
				&self.calls
			}
		}
		impl Encode for BatchAll {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.length).encode_to(dest);
				dest.write(&self.calls);
			}
		}
		impl Decode for BatchAll {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let length = Compact::<u32>::decode(input)?.0;
				let calls = decode_already_decoded(input)?;
				Ok(Self { length, calls })
			}
		}
		impl HasHeader for BatchAll {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Default, Clone)]
		pub struct ForceBatch {
			pub length: u32,
			pub calls: Vec<u8>,
		}
		impl ForceBatch {
			pub fn new() -> Self {
				Self::default()
			}

			pub fn decode_calls(&self) -> Result<Vec<RuntimeCall>, codec::Error> {
				if self.length == 0 {
					return Ok(Vec::new());
				}

				let mut runtime_calls: Vec<RuntimeCall> = Vec::with_capacity(self.length as usize);
				let mut calls = self.calls.as_slice();
				for _ in 0..self.length {
					runtime_calls.push(RuntimeCall::decode(&mut calls)?)
				}
				if !calls.is_empty() {
					return Err(codec::Error::from(
						"Bytes left in array. Failed to decode Batch call into RuntimeCalls",
					));
				}

				Ok(runtime_calls)
			}

			pub fn add_calls(&mut self, value: Vec<ExtrinsicCall>) {
				for v in value {
					self.add_call(v);
				}
			}

			pub fn add_call(&mut self, value: ExtrinsicCall) {
				self.length += 1;
				value.encode_to(&mut self.calls);
			}

			pub fn add_hex(&mut self, value: &str) -> Result<(), const_hex::FromHexError> {
				let decoded = const_hex::decode(value.trim_start_matches("0x"))?;
				self.add(decoded);
				Ok(())
			}

			pub fn add(&mut self, mut value: Vec<u8>) {
				self.length += 1;
				self.calls.append(&mut value);
			}

			pub fn len(&self) -> u32 {
				self.length
			}

			pub fn is_empty(&self) -> bool {
				self.len() == 0
			}

			pub fn calls(&self) -> &[u8] {
				&self.calls
			}
		}
		impl Encode for ForceBatch {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.length).encode_to(dest);
				dest.write(&self.calls);
			}
		}
		impl Decode for ForceBatch {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let length = Compact::<u32>::decode(input)?.0;
				let calls = decode_already_decoded(input)?;
				Ok(Self { length, calls })
			}
		}
		impl HasHeader for ForceBatch {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
	}
}

pub mod nomination_pools {
	use super::*;
	pub const PALLET_ID: u8 = 36;

	pub mod types {
		use super::*;

		#[derive(Debug, Clone)]
		pub enum ClaimPermission {
			Permissioned,
			PermissionlessCompound,
			PermissionlessWithdraw,
			PermissionlessAll,
		}
		impl Encode for ClaimPermission {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				match self {
					Self::Permissioned => 0u8.encode_to(dest),
					Self::PermissionlessCompound => 1u8.encode_to(dest),
					Self::PermissionlessWithdraw => 2u8.encode_to(dest),
					Self::PermissionlessAll => 3u8.encode_to(dest),
				}
			}
		}
		impl Decode for ClaimPermission {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::Permissioned),
					1 => Ok(Self::PermissionlessCompound),
					2 => Ok(Self::PermissionlessWithdraw),
					3 => Ok(Self::PermissionlessAll),
					_ => Err("Failed to decode ClaimPermission. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone)]
		pub enum BondExtraValue {
			FreBalance(u128),
			Rewards,
		}
		impl Encode for BondExtraValue {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				match self {
					Self::FreBalance(v) => {
						0u8.encode_to(dest);
						v.encode_to(dest);
					},
					Self::Rewards => 1u8.encode_to(dest),
				}
			}
		}
		impl Decode for BondExtraValue {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::FreBalance(Decode::decode(input)?)),
					1 => Ok(Self::Rewards),
					_ => Err("Failed to decode BondExtra. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone)]
		pub enum PoolState {
			Open,
			Blocked,
			Destroying,
		}
		impl Encode for PoolState {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				match self {
					Self::Open => 0u8.encode_to(dest),
					Self::Blocked => 1u8.encode_to(dest),
					Self::Destroying => 2u8.encode_to(dest),
				}
			}
		}
		impl Decode for PoolState {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::Open),
					1 => Ok(Self::Blocked),
					2 => Ok(Self::Destroying),
					_ => Err("Failed to decode PoolState. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone)]
		pub enum ConfigOpAccount {
			Noop,
			Set(AccountId),
			Remove,
		}
		impl Encode for ConfigOpAccount {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				match self {
					Self::Noop => 1u8.encode_to(dest),
					Self::Set(v) => {
						1u8.encode_to(dest);
						v.encode_to(dest);
					},
					Self::Remove => 2u8.encode_to(dest),
				}
			}
		}
		impl Decode for ConfigOpAccount {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::Noop),
					1 => Ok(Self::Set(Decode::decode(input)?)),
					2 => Ok(Self::Remove),
					_ => Err("Failed to decode ConfigOpAccount. Unknown variant".into()),
				}
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct BondExtra {
			pub value: super::types::BondExtraValue,
		}
		impl Encode for BondExtra {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.value.encode_to(dest);
			}
		}
		impl Decode for BondExtra {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Decode::decode(input)?;
				Ok(Self { value })
			}
		}
		impl HasHeader for BondExtra {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Debug, Clone)]
		pub struct BondExtraOther {
			pub member: MultiAddress,
			pub value: super::types::BondExtraValue,
		}
		impl Encode for BondExtraOther {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.member.encode_to(dest);
				self.value.encode_to(dest);
			}
		}
		impl Decode for BondExtraOther {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let member = Decode::decode(input)?;
				let value = Decode::decode(input)?;
				Ok(Self { member, value })
			}
		}
		impl HasHeader for BondExtraOther {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 14);
		}

		#[derive(Debug, Clone)]
		pub struct Chill {
			pub pool_id: u32,
		}
		impl Encode for Chill {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.pool_id.encode_to(dest);
			}
		}
		impl Decode for Chill {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self { pool_id: Decode::decode(input)? })
			}
		}
		impl HasHeader for Chill {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 13);
		}

		#[derive(Debug, Clone)]
		pub struct ClaimCommission {
			pub pool_id: u32,
		}
		impl Encode for ClaimCommission {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.pool_id.encode_to(dest);
			}
		}
		impl Decode for ClaimCommission {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let pool_id = Decode::decode(input)?;
				Ok(Self { pool_id })
			}
		}
		impl HasHeader for ClaimCommission {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 20);
		}

		#[derive(Debug, Clone)]
		pub struct ClaimPayout {}
		impl Encode for ClaimPayout {
			fn encode_to<T: codec::Output + ?Sized>(&self, _dest: &mut T) {}
		}
		impl Decode for ClaimPayout {
			fn decode<I: codec::Input>(_input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self {})
			}
		}
		impl HasHeader for ClaimPayout {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Clone)]
		pub struct ClaimPayoutOther {
			pub owner: AccountId,
		}
		impl Encode for ClaimPayoutOther {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.owner.encode_to(dest);
			}
		}
		impl Decode for ClaimPayoutOther {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let owner = Decode::decode(input)?;
				Ok(Self { owner })
			}
		}
		impl HasHeader for ClaimPayoutOther {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 16);
		}

		#[derive(Debug, Clone)]
		pub struct Create {
			pub amount: u128,
			pub root: MultiAddress,
			pub nominator: MultiAddress,
			pub bouncer: MultiAddress,
		}
		impl Encode for Create {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.amount).encode_to(dest);
				self.root.encode_to(dest);
				self.nominator.encode_to(dest);
				self.bouncer.encode_to(dest);
			}
		}
		impl Decode for Create {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let amount = Compact::<u128>::decode(input)?.0;
				let root = Decode::decode(input)?;
				let nominator = Decode::decode(input)?;
				let bouncer = Decode::decode(input)?;
				Ok(Self { amount, root, nominator, bouncer })
			}
		}
		impl HasHeader for Create {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 6);
		}

		#[derive(Debug, Clone)]
		pub struct CreateWithPoolId {
			pub amount: u128,
			pub root: MultiAddress,
			pub nominator: MultiAddress,
			pub bouncer: MultiAddress,
			pub pool_id: u32,
		}
		impl Encode for CreateWithPoolId {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.amount).encode_to(dest);
				self.root.encode_to(dest);
				self.nominator.encode_to(dest);
				self.bouncer.encode_to(dest);
				self.pool_id.encode_to(dest);
			}
		}
		impl Decode for CreateWithPoolId {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let amount = Compact::<u128>::decode(input)?.0;
				let root = Decode::decode(input)?;
				let nominator = Decode::decode(input)?;
				let bouncer = Decode::decode(input)?;
				let pool_id = Decode::decode(input)?;
				Ok(Self { amount, root, nominator, bouncer, pool_id })
			}
		}
		impl HasHeader for CreateWithPoolId {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 7);
		}

		#[derive(Debug, Clone)]
		pub struct Join {
			pub amount: u128,
			pub pool_id: u32,
		}
		impl Encode for Join {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.amount).encode_to(dest);
				self.pool_id.encode_to(dest);
			}
		}
		impl Decode for Join {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let amount = Compact::<u128>::decode(input)?.0;
				let pool_id = Decode::decode(input)?;
				Ok(Self { amount, pool_id })
			}
		}
		impl HasHeader for Join {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct Nominate {
			pub pool_id: u32,
			pub validators: Vec<AccountId>,
		}
		impl Encode for Nominate {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.pool_id.encode_to(dest);
				self.validators.encode_to(dest);
			}
		}
		impl Decode for Nominate {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let pool_id = Decode::decode(input)?;
				let validators = Decode::decode(input)?;
				Ok(Self { pool_id, validators })
			}
		}
		impl HasHeader for Nominate {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 8);
		}

		#[derive(Debug, Clone)]
		pub struct SetClaimPermission {
			pub permission: types::ClaimPermission,
		}
		impl Encode for SetClaimPermission {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.permission.encode_to(dest);
			}
		}
		impl Decode for SetClaimPermission {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let permission = Decode::decode(input)?;
				Ok(Self { permission })
			}
		}
		impl HasHeader for SetClaimPermission {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 15);
		}

		#[derive(Debug, Clone)]
		pub struct SetCommission {
			pub pool_id: u32,
			pub new_commission: Option<(u32, AccountId)>,
		}
		impl Encode for SetCommission {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.pool_id.encode_to(dest);
				self.new_commission.encode_to(dest);
			}
		}
		impl Decode for SetCommission {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let pool_id = Decode::decode(input)?;
				let new_commission = Decode::decode(input)?;
				Ok(Self { pool_id, new_commission })
			}
		}
		impl HasHeader for SetCommission {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 17);
		}

		#[derive(Debug, Clone)]
		pub struct SetCommissionChangeRate {
			pub pool_id: u32,
			pub max_increase: u32,
			pub min_delay: u32,
		}
		impl Encode for SetCommissionChangeRate {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.pool_id.encode_to(dest);
				self.max_increase.encode_to(dest);
				self.min_delay.encode_to(dest);
			}
		}
		impl Decode for SetCommissionChangeRate {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let pool_id = Decode::decode(input)?;
				let max_increase = Decode::decode(input)?;
				let min_delay = Decode::decode(input)?;
				Ok(Self { pool_id, max_increase, min_delay })
			}
		}
		impl HasHeader for SetCommissionChangeRate {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 19);
		}

		#[derive(Debug, Clone)]
		pub struct SetCommissionMax {
			pub pool_id: u32,
			pub max_commission: u32,
		}
		impl Encode for SetCommissionMax {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.pool_id.encode_to(dest);
				self.max_commission.encode_to(dest);
			}
		}
		impl Decode for SetCommissionMax {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let pool_id = Decode::decode(input)?;
				let max_commission = Decode::decode(input)?;
				Ok(Self { pool_id, max_commission })
			}
		}
		impl HasHeader for SetCommissionMax {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 18);
		}

		#[derive(Debug, Clone)]
		pub struct SetMetadata {
			pub pool_id: u32,
			pub metadata: Vec<u8>,
		}
		impl Encode for SetMetadata {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.pool_id.encode_to(dest);
				self.metadata.encode_to(dest);
			}
		}
		impl Decode for SetMetadata {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let pool_id = Decode::decode(input)?;
				let metadata = Decode::decode(input)?;
				Ok(Self { pool_id, metadata })
			}
		}
		impl HasHeader for SetMetadata {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 10);
		}

		#[derive(Debug, Clone)]
		pub struct SetState {
			pub pool_id: u32,
			pub state: types::PoolState,
		}
		impl Encode for SetState {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.pool_id.encode_to(dest);
				self.state.encode_to(dest);
			}
		}
		impl Decode for SetState {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let pool_id = Decode::decode(input)?;
				let state = Decode::decode(input)?;
				Ok(Self { pool_id, state })
			}
		}
		impl HasHeader for SetState {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 9);
		}

		#[derive(Debug, Clone)]
		pub struct Unbond {
			pub member_account: MultiAddress,
			pub unbonding_points: u128,
		}
		impl Encode for Unbond {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.member_account.encode_to(dest);
				Compact(self.unbonding_points).encode_to(dest);
			}
		}
		impl Decode for Unbond {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let member_account = Decode::decode(input)?;
				let unbonding_points = Compact::<u128>::decode(input)?.0;
				Ok(Self { member_account, unbonding_points })
			}
		}
		impl HasHeader for Unbond {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Debug, Clone)]
		pub struct UpdateRoles {
			pub pool_id: u32,
			pub new_root: types::ConfigOpAccount,
			pub new_nominator: types::ConfigOpAccount,
			pub new_bouncer: types::ConfigOpAccount,
		}
		impl Encode for UpdateRoles {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.pool_id.encode_to(dest);
				self.new_root.encode_to(dest);
				self.new_nominator.encode_to(dest);
				self.new_bouncer.encode_to(dest);
			}
		}
		impl Decode for UpdateRoles {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let pool_id = Decode::decode(input)?;
				let new_root = Decode::decode(input)?;
				let new_nominator = Decode::decode(input)?;
				let new_bouncer = Decode::decode(input)?;
				Ok(Self { pool_id, new_root, new_nominator, new_bouncer })
			}
		}
		impl HasHeader for UpdateRoles {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 12);
		}

		#[derive(Debug, Clone)]
		pub struct WithdrawUnbonded {
			pub member_account: MultiAddress,
			pub num_slashing_spans: u32,
		}
		impl Encode for WithdrawUnbonded {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.member_account.encode_to(dest);
				self.num_slashing_spans.encode_to(dest);
			}
		}
		impl Decode for WithdrawUnbonded {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let member_account = Decode::decode(input)?;
				let num_slashing_spans = Decode::decode(input)?;
				Ok(Self { member_account, num_slashing_spans })
			}
		}
		impl HasHeader for WithdrawUnbonded {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 5);
		}
	}
}

pub mod proxy {
	use super::*;
	pub const PALLET_ID: u8 = 40;

	pub mod types {
		use super::*;

		#[derive(Debug, Clone, Copy)]
		#[repr(u8)]
		pub enum ProxyType {
			Any = 0,
			NonTransfer = 1,
			Governance = 2,
			Staking = 3,
			IdentityJudgement = 4,
			NominationPools = 5,
		}
		impl Encode for ProxyType {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = *self as u8;
				variant.encode_to(dest);
			}
		}
		impl Decode for ProxyType {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::Any),
					1 => Ok(Self::NonTransfer),
					2 => Ok(Self::Governance),
					3 => Ok(Self::Staking),
					4 => Ok(Self::IdentityJudgement),
					5 => Ok(Self::NominationPools),
					_ => Err("Failed to decode ProxyType. Unknown variant".into()),
				}
			}
		}
	}

	pub mod events {
		use super::*;

		/// A proxy was executed correctly, with the given.
		#[derive(Debug, Clone)]
		pub struct ProxyExecuted {
			pub result: Result<(), super::system::types::DispatchError>,
		}
		impl HasHeader for ProxyExecuted {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}
		impl Encode for ProxyExecuted {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.result.encode_to(dest);
			}
		}
		impl Decode for ProxyExecuted {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let result = Decode::decode(input)?;
				Ok(Self { result })
			}
		}

		/// A pure account has been created by new proxy with given
		/// disambiguation index and proxy type.
		#[derive(Debug, Clone)]
		pub struct PureCreated {
			pub pure: AccountId,
			pub who: AccountId,
			pub proxy_type: super::types::ProxyType,
			pub disambiguation_index: u16,
		}
		impl HasHeader for PureCreated {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
		impl Encode for PureCreated {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.pure.encode_to(dest);
				self.who.encode_to(dest);
				self.proxy_type.encode_to(dest);
				self.disambiguation_index.encode_to(dest);
			}
		}
		impl Decode for PureCreated {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let pure = Decode::decode(input)?;
				let who = Decode::decode(input)?;
				let proxy_type = Decode::decode(input)?;
				let disambiguation_index = Decode::decode(input)?;
				Ok(Self { pure, who, proxy_type, disambiguation_index })
			}
		}

		/// An announcement was placed to make a call in the future.
		#[derive(Debug, Clone)]
		pub struct Announced {
			pub real: AccountId,
			pub proxy: AccountId,
			pub call_hash: H256,
		}
		impl HasHeader for Announced {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}
		impl Encode for Announced {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.real.encode_to(dest);
				self.proxy.encode_to(dest);
				self.call_hash.encode_to(dest);
			}
		}
		impl Decode for Announced {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let real = Decode::decode(input)?;
				let proxy = Decode::decode(input)?;
				let call_hash = Decode::decode(input)?;
				Ok(Self { real, proxy, call_hash })
			}
		}

		/// A proxy was added.
		#[derive(Debug, Clone)]
		pub struct ProxyAdded {
			pub delegator: AccountId,
			pub delegatee: AccountId,
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
		}
		impl HasHeader for ProxyAdded {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 3);
		}
		impl Encode for ProxyAdded {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.delegator.encode_to(dest);
				self.delegatee.encode_to(dest);
				self.proxy_type.encode_to(dest);
				self.delay.encode_to(dest);
			}
		}
		impl Decode for ProxyAdded {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let delegator = Decode::decode(input)?;
				let delegatee = Decode::decode(input)?;
				let proxy_type = Decode::decode(input)?;
				let delay = Decode::decode(input)?;
				Ok(Self { delegator, delegatee, proxy_type, delay })
			}
		}

		/// A proxy was removed.
		#[derive(Debug, Clone)]
		pub struct ProxyRemoved {
			pub delegator: AccountId,
			pub delegatee: AccountId,
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
		}
		impl HasHeader for ProxyRemoved {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
		impl Encode for ProxyRemoved {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.delegator.encode_to(dest);
				self.delegatee.encode_to(dest);
				self.proxy_type.encode_to(dest);
				self.delay.encode_to(dest);
			}
		}
		impl Decode for ProxyRemoved {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let delegator = Decode::decode(input)?;
				let delegatee = Decode::decode(input)?;
				let proxy_type = Decode::decode(input)?;
				let delay = Decode::decode(input)?;
				Ok(Self { delegator, delegatee, proxy_type, delay })
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct Proxy {
			pub id: MultiAddress,
			pub force_proxy_type: Option<super::types::ProxyType>,
			pub call: ExtrinsicCall,
		}
		impl Encode for Proxy {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.id.encode());
				dest.write(&self.force_proxy_type.encode());
				dest.write(&self.call.encode());
			}
		}
		impl Decode for Proxy {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let id = Decode::decode(input)?;
				let force_proxy_type = Decode::decode(input)?;
				let call = Decode::decode(input)?;
				Ok(Self { id, force_proxy_type, call })
			}
		}
		impl HasHeader for Proxy {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct AddProxy {
			pub id: MultiAddress,
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
		}
		impl Encode for AddProxy {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.id.encode());
				dest.write(&self.proxy_type.encode());
				dest.write(&self.delay.encode());
			}
		}
		impl Decode for AddProxy {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let id = Decode::decode(input)?;
				let proxy_type = Decode::decode(input)?;
				let delay = Decode::decode(input)?;
				Ok(Self { id, proxy_type, delay })
			}
		}
		impl HasHeader for AddProxy {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Debug, Clone)]
		pub struct RemoveProxy {
			pub delegate: MultiAddress,
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
		}
		impl Encode for RemoveProxy {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.delegate.encode());
				dest.write(&self.proxy_type.encode());
				dest.write(&self.delay.encode());
			}
		}
		impl Decode for RemoveProxy {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let delegate = Decode::decode(input)?;
				let proxy_type = Decode::decode(input)?;
				let delay = Decode::decode(input)?;
				Ok(Self { delegate, proxy_type, delay })
			}
		}
		impl HasHeader for RemoveProxy {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Clone)]
		pub struct RemoveProxies;
		impl Encode for RemoveProxies {
			fn encode_to<T: codec::Output + ?Sized>(&self, _dest: &mut T) {}
		}
		impl Decode for RemoveProxies {
			fn decode<I: codec::Input>(_input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self)
			}
		}
		impl HasHeader for RemoveProxies {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Debug, Clone)]
		pub struct CreatePure {
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
			pub index: u16,
		}
		impl Encode for CreatePure {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.proxy_type.encode());
				dest.write(&self.delay.encode());
				dest.write(&self.index.encode());
			}
		}
		impl Decode for CreatePure {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let proxy_type = Decode::decode(input)?;
				let delay = Decode::decode(input)?;
				let index = Decode::decode(input)?;
				Ok(Self { proxy_type, delay, index })
			}
		}
		impl HasHeader for CreatePure {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 4);
		}

		#[derive(Debug, Clone)]
		pub struct KillPure {
			pub spawner: MultiAddress,
			pub proxy_type: super::types::ProxyType,
			pub index: u16,
			pub height: u32,    // Compact
			pub ext_index: u32, // Compact
		}
		impl Encode for KillPure {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.spawner.encode_to(dest);
				self.proxy_type.encode_to(dest);
				self.index.encode_to(dest);
				Compact::<u32>(self.height).encode_to(dest);
				Compact::<u32>(self.ext_index).encode_to(dest);
			}
		}
		impl Decode for KillPure {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let spawner = Decode::decode(input)?;
				let proxy_type = Decode::decode(input)?;
				let index = Decode::decode(input)?;
				let height = Compact::<u32>::decode(input)?.0;
				let ext_index = Compact::<u32>::decode(input)?.0;
				Ok(Self { spawner, proxy_type, index, height, ext_index })
			}
		}
		impl HasHeader for KillPure {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 5);
		}
	}
}

pub mod multisig {
	use super::*;
	pub const PALLET_ID: u8 = 34;

	pub mod types {
		use super::*;
		pub use crate::types::substrate::Weight;

		#[derive(Debug, Clone, Copy)]
		pub struct Timepoint {
			pub height: u32,
			pub index: u32,
		}
		impl Encode for Timepoint {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.height.encode_to(dest);
				self.index.encode_to(dest);
			}
		}
		impl Decode for Timepoint {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let height = Decode::decode(input)?;
				let index = Decode::decode(input)?;
				Ok(Self { height, index })
			}
		}
	}

	pub mod events {
		use super::*;

		/// A new multisig operation has begun.
		#[derive(Debug, Clone)]
		pub struct NewMultisig {
			pub approving: AccountId,
			pub multisig: AccountId,
			pub call_hash: H256,
		}
		impl HasHeader for NewMultisig {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}
		impl Encode for NewMultisig {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.approving.encode_to(dest);
				self.multisig.encode_to(dest);
				self.call_hash.encode_to(dest);
			}
		}
		impl Decode for NewMultisig {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let approving = Decode::decode(input)?;
				let multisig = Decode::decode(input)?;
				let call_hash = Decode::decode(input)?;
				Ok(Self { approving, multisig, call_hash })
			}
		}

		/// A multisig operation has been approved by someone.
		#[derive(Debug, Clone)]
		pub struct MultisigApproval {
			pub approving: AccountId,
			pub timepoint: super::types::Timepoint,
			pub multisig: AccountId,
			pub call_hash: H256,
		}
		impl HasHeader for MultisigApproval {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
		impl Encode for MultisigApproval {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.approving.encode_to(dest);
				self.timepoint.encode_to(dest);
				self.multisig.encode_to(dest);
				self.call_hash.encode_to(dest);
			}
		}
		impl Decode for MultisigApproval {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let approving = Decode::decode(input)?;
				let timepoint = Decode::decode(input)?;
				let multisig = Decode::decode(input)?;
				let call_hash = Decode::decode(input)?;
				Ok(Self { approving, timepoint, multisig, call_hash })
			}
		}

		/// A multisig operation has been executed.
		#[derive(Debug, Clone)]
		pub struct MultisigExecuted {
			pub approving: AccountId,
			pub timepoint: super::types::Timepoint,
			pub multisig: AccountId,
			pub call_hash: H256,
			pub result: Result<(), super::system::types::DispatchError>,
		}
		impl HasHeader for MultisigExecuted {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}
		impl Encode for MultisigExecuted {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.approving.encode_to(dest);
				self.timepoint.encode_to(dest);
				self.multisig.encode_to(dest);
				self.call_hash.encode_to(dest);
				self.result.encode_to(dest);
			}
		}
		impl Decode for MultisigExecuted {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let approving = Decode::decode(input)?;
				let timepoint = Decode::decode(input)?;
				let multisig = Decode::decode(input)?;
				let call_hash = Decode::decode(input)?;
				let result = Decode::decode(input)?;
				Ok(Self { approving, timepoint, multisig, call_hash, result })
			}
		}

		/// A multisig operation has been cancelled.
		#[derive(Debug, Clone)]
		pub struct MultisigCancelled {
			pub cancelling: AccountId,
			pub timepoint: super::types::Timepoint,
			pub multisig: AccountId,
			pub call_hash: H256,
		}
		impl HasHeader for MultisigCancelled {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 3);
		}
		impl Encode for MultisigCancelled {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.cancelling.encode_to(dest);
				self.timepoint.encode_to(dest);
				self.multisig.encode_to(dest);
				self.call_hash.encode_to(dest);
			}
		}
		impl Decode for MultisigCancelled {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let cancelling = Decode::decode(input)?;
				let timepoint = Decode::decode(input)?;
				let multisig = Decode::decode(input)?;
				let call_hash = Decode::decode(input)?;
				Ok(Self { cancelling, timepoint, multisig, call_hash })
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct AsMultiThreshold1 {
			pub other_signatories: Vec<AccountId>,
			pub call: ExtrinsicCall,
		}
		impl Encode for AsMultiThreshold1 {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.other_signatories.encode_to(dest);
				self.call.encode_to(dest);
			}
		}
		impl Decode for AsMultiThreshold1 {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let other_signatories = Decode::decode(input)?;
				let call = Decode::decode(input)?;
				Ok(Self { other_signatories, call })
			}
		}
		impl HasHeader for AsMultiThreshold1 {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct AsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub maybe_timepoint: Option<super::types::Timepoint>,
			pub call: ExtrinsicCall,
			pub max_weight: super::types::Weight,
		}
		impl Encode for AsMulti {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.threshold.encode_to(dest);
				self.other_signatories.encode_to(dest);
				self.maybe_timepoint.encode_to(dest);
				self.call.encode_to(dest);
				self.max_weight.encode_to(dest);
			}
		}
		impl Decode for AsMulti {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let threshold = Decode::decode(input)?;
				let other_signatories = Decode::decode(input)?;
				let maybe_timepoint = Decode::decode(input)?;
				let call = RuntimeCall::decode(input)?;
				let max_weight = Decode::decode(input)?;

				let encoded_call = call.encode();
				Ok(Self {
					threshold,
					other_signatories,
					maybe_timepoint,
					call: ExtrinsicCall::decode(&mut encoded_call.as_slice())?,
					max_weight,
				})
			}
		}
		impl HasHeader for AsMulti {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Debug, Clone)]
		pub struct ApproveAsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub maybe_timepoint: Option<super::types::Timepoint>,
			pub call_hash: H256,
			pub max_weight: super::types::Weight,
		}
		impl Encode for ApproveAsMulti {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.threshold.encode_to(dest);
				self.other_signatories.encode_to(dest);
				self.maybe_timepoint.encode_to(dest);
				self.call_hash.encode_to(dest);
				self.max_weight.encode_to(dest);
			}
		}
		impl Decode for ApproveAsMulti {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let threshold = Decode::decode(input)?;
				let other_signatories = Decode::decode(input)?;
				let maybe_timepoint = Decode::decode(input)?;
				let call_hash = Decode::decode(input)?;
				let max_weight = Decode::decode(input)?;
				Ok(Self {
					threshold,
					other_signatories,
					maybe_timepoint,
					call_hash,
					max_weight,
				})
			}
		}
		impl HasHeader for ApproveAsMulti {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Clone)]
		pub struct CancelAsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub timepoint: super::types::Timepoint,
			pub call_hash: H256,
		}
		impl Encode for CancelAsMulti {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.threshold.encode_to(dest);
				self.other_signatories.encode_to(dest);
				self.timepoint.encode_to(dest);
				self.call_hash.encode_to(dest);
			}
		}
		impl Decode for CancelAsMulti {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let threshold = Decode::decode(input)?;
				let other_signatories = Decode::decode(input)?;
				let timepoint = Decode::decode(input)?;
				let call_hash = Decode::decode(input)?;
				Ok(Self { threshold, other_signatories, timepoint, call_hash })
			}
		}
		impl HasHeader for CancelAsMulti {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 3);
		}
	}
}

pub mod vector {
	use super::*;
	pub const PALLET_ID: u8 = 39;

	pub mod types {
		use super::*;
		pub use crate::types::substrate::Weight;

		/// Message type used to bridge between Avail & other chains
		#[derive(Debug, Clone, Serialize, Deserialize)]
		#[serde(rename_all = "camelCase")]
		pub struct AddressedMessage {
			pub message: Message,
			pub from: H256,
			pub to: H256,
			pub origin_domain: u32,      // Compact
			pub destination_domain: u32, // Compact
			/// Unique identifier for the message
			pub id: u64, // Compact
		}
		impl Encode for AddressedMessage {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.message.encode_to(dest);
				self.from.encode_to(dest);
				self.to.encode_to(dest);
				Compact(self.origin_domain).encode_to(dest);
				Compact(self.destination_domain).encode_to(dest);
				Compact(self.id).encode_to(dest);
			}
		}
		impl Decode for AddressedMessage {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let message = Decode::decode(input)?;
				let from = Decode::decode(input)?;
				let to = Decode::decode(input)?;
				let origin_domain = Compact::<u32>::decode(input)?.0;
				let destination_domain = Compact::<u32>::decode(input)?.0;
				let id = Compact::<u64>::decode(input)?.0;
				Ok(Self { message, from, to, origin_domain, destination_domain, id })
			}
		}

		/// Possible types of Messages allowed by Avail to bridge to other chains.
		#[derive(Debug, Clone, Serialize, Deserialize)]
		#[repr(u8)]
		pub enum Message {
			ArbitraryMessage(Vec<u8>) = 0,
			FungibleToken { asset_id: H256, amount: u128 } = 1,
		}
		impl Encode for Message {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Message::ArbitraryMessage(items) => {
						items.encode_to(dest);
					},
					Message::FungibleToken { asset_id, amount } => {
						asset_id.encode_to(dest);
						Compact(*amount).encode_to(dest);
					},
				}
			}
		}
		impl Decode for Message {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => {
						let items = Decode::decode(input)?;
						Ok(Self::ArbitraryMessage(items))
					},
					1 => {
						let asset_id = Decode::decode(input)?;
						let amount = Compact::<u128>::decode(input)?.0;
						Ok(Self::FungibleToken { asset_id, amount })
					},
					_ => Err("Failed to decode Message. Unknown Message variant".into()),
				}
			}
		}

		#[derive(Debug, Clone)]
		pub struct Configuration {
			pub slots_per_period: Compact<u64>,
			pub finality_threshold: Compact<u16>,
		}
		impl Encode for Configuration {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.slots_per_period.encode_to(dest);
				self.finality_threshold.encode_to(dest);
			}
		}
		impl Decode for Configuration {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let slots_per_period = Decode::decode(input)?;
				let finality_threshold = Decode::decode(input)?;
				Ok(Self { slots_per_period, finality_threshold })
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct FulfillCall {
			pub function_id: H256,
			pub input: Vec<u8>,
			pub output: Vec<u8>,
			pub proof: Vec<u8>,
			pub slot: u64, // Compact
		}
		impl Encode for FulfillCall {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.function_id.encode_to(dest);
				self.input.encode_to(dest);
				self.output.encode_to(dest);
				self.proof.encode_to(dest);
				Compact(self.slot).encode_to(dest);
			}
		}
		impl Decode for FulfillCall {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let function_id = Decode::decode(input)?;
				let inputt = Decode::decode(input)?;
				let output = Decode::decode(input)?;
				let proof = Decode::decode(input)?;
				let slot = Compact::<u64>::decode(input)?.0;
				Ok(Self { function_id, input: inputt, output, proof, slot })
			}
		}
		impl HasHeader for FulfillCall {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct Execute {
			pub slot: u64, // Compact
			pub addr_message: super::types::AddressedMessage,
			pub account_proof: Vec<Vec<u8>>,
			pub storage_proof: Vec<Vec<u8>>,
		}
		impl Encode for Execute {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.slot).encode_to(dest);
				self.addr_message.encode_to(dest);
				self.account_proof.encode_to(dest);
				self.storage_proof.encode_to(dest);
			}
		}
		impl Decode for Execute {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let slot = Compact::<u64>::decode(input)?.0;
				let addr_message = Decode::decode(input)?;
				let account_proof = Decode::decode(input)?;
				let storage_proof = Decode::decode(input)?;
				Ok(Self { slot, addr_message, account_proof, storage_proof })
			}
		}
		impl HasHeader for Execute {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Debug, Clone)]
		pub struct SourceChainFroze {
			pub source_chain_id: u32, // Compact
			pub frozen: bool,
		}
		impl Encode for SourceChainFroze {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.source_chain_id).encode_to(dest);
				self.frozen.encode_to(dest);
			}
		}
		impl Decode for SourceChainFroze {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let source_chain_id = Compact::<u32>::decode(input)?.0;
				let frozen = Decode::decode(input)?;
				Ok(Self { source_chain_id, frozen })
			}
		}
		impl HasHeader for SourceChainFroze {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Clone)]
		pub struct SendMessage {
			pub message: types::Message,
			pub to: H256,
			pub domain: u32, // Compact
		}
		impl Encode for SendMessage {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.message.encode_to(dest);
				self.to.encode_to(dest);
				Compact(self.domain).encode_to(dest);
			}
		}
		impl Decode for SendMessage {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let message = Decode::decode(input)?;
				let to = Decode::decode(input)?;
				let domain = Compact::<u32>::decode(input)?.0;
				Ok(Self { message, to, domain })
			}
		}
		impl HasHeader for SendMessage {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Debug, Clone)]
		pub struct SetPoseidonHash {
			pub period: Compact<u64>,
			pub poseidon_hash: Vec<u8>,
		}
		impl Encode for SetPoseidonHash {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.period.encode_to(dest);
				self.poseidon_hash.encode_to(dest);
			}
		}
		impl Decode for SetPoseidonHash {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let period = Decode::decode(input)?;
				let poseidon_hash = Decode::decode(input)?;
				Ok(Self { period, poseidon_hash })
			}
		}
		impl HasHeader for SetPoseidonHash {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 4);
		}

		#[derive(Debug, Clone)]
		pub struct SetBroadcaster {
			pub broadcaster_domain: Compact<u32>,
			pub broadcaster: H256,
		}
		impl Encode for SetBroadcaster {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.broadcaster_domain.encode_to(dest);
				self.broadcaster.encode_to(dest);
			}
		}
		impl Decode for SetBroadcaster {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let broadcaster_domain = Decode::decode(input)?;
				let broadcaster = Decode::decode(input)?;
				Ok(Self { broadcaster_domain, broadcaster })
			}
		}
		impl HasHeader for SetBroadcaster {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 5);
		}

		#[derive(Debug, Clone)]
		pub struct SetWhitelistedDomains {
			pub value: Vec<u32>,
		}
		impl Encode for SetWhitelistedDomains {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.value.encode());
			}
		}
		impl Decode for SetWhitelistedDomains {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Decode::decode(input)?;
				Ok(Self { value })
			}
		}
		impl HasHeader for SetWhitelistedDomains {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 6);
		}

		#[derive(Debug, Clone)]
		pub struct SetConfiguration {
			pub value: super::types::Configuration,
		}
		impl Encode for SetConfiguration {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.value.encode());
			}
		}
		impl Decode for SetConfiguration {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Decode::decode(input)?;
				Ok(Self { value })
			}
		}
		impl HasHeader for SetConfiguration {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 7);
		}

		#[derive(Debug, Clone)]
		pub struct SetFunctionIds {
			pub value: Option<(H256, H256)>,
		}
		impl Encode for SetFunctionIds {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.value.encode());
			}
		}
		impl Decode for SetFunctionIds {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Decode::decode(input)?;
				Ok(Self { value })
			}
		}
		impl HasHeader for SetFunctionIds {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 8);
		}

		#[derive(Debug, Clone)]
		pub struct SetStepVerificationKey {
			pub value: Option<Vec<u8>>,
		}
		impl Encode for SetStepVerificationKey {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.value.encode());
			}
		}
		impl Decode for SetStepVerificationKey {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Decode::decode(input)?;
				Ok(Self { value })
			}
		}
		impl HasHeader for SetStepVerificationKey {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 9);
		}

		#[derive(Debug, Clone)]
		pub struct SetRotateVerificationKey {
			pub value: Option<Vec<u8>>,
		}
		impl Encode for SetRotateVerificationKey {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.value.encode());
			}
		}
		impl Decode for SetRotateVerificationKey {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Decode::decode(input)?;
				Ok(Self { value })
			}
		}
		impl HasHeader for SetRotateVerificationKey {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 10);
		}

		#[derive(Debug, Clone)]
		pub struct SetUpdater {
			pub updater: H256,
		}
		impl Encode for SetUpdater {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.updater.encode());
			}
		}
		impl Decode for SetUpdater {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let updater = Decode::decode(input)?;
				Ok(Self { updater })
			}
		}
		impl HasHeader for SetUpdater {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 12);
		}

		#[derive(Debug, Clone)]
		pub struct Fulfill {
			pub proof: Vec<u8>,
			pub public_values: Vec<u8>,
		}
		impl Encode for Fulfill {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.proof.encode());
				dest.write(&self.public_values.encode());
			}
		}
		impl Decode for Fulfill {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let proof = Decode::decode(input)?;
				let public_values = Decode::decode(input)?;
				Ok(Self { proof, public_values })
			}
		}
		impl HasHeader for Fulfill {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 13);
		}

		#[derive(Debug, Clone)]
		pub struct SetSp1VerificationKey {
			pub sp1_vk: H256,
		}
		impl Encode for SetSp1VerificationKey {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.sp1_vk.encode());
			}
		}
		impl Decode for SetSp1VerificationKey {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let sp1_vk = Decode::decode(input)?;
				Ok(Self { sp1_vk })
			}
		}
		impl HasHeader for SetSp1VerificationKey {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 14);
		}

		#[derive(Debug, Clone)]
		pub struct SetSyncCommitteeHash {
			pub period: u64,
			pub hash: H256,
		}
		impl Encode for SetSyncCommitteeHash {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.period.encode());
				dest.write(&self.hash.encode());
			}
		}
		impl Decode for SetSyncCommitteeHash {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let period = Decode::decode(input)?;
				let hash = Decode::decode(input)?;
				Ok(Self { period, hash })
			}
		}
		impl HasHeader for SetSyncCommitteeHash {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 15);
		}

		#[derive(Debug, Clone)]
		pub struct EnableMock {
			pub value: bool,
		}
		impl Encode for EnableMock {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.value.encode());
			}
		}
		impl Decode for EnableMock {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Decode::decode(input)?;
				Ok(Self { value })
			}
		}
		impl HasHeader for EnableMock {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 16);
		}

		#[derive(Debug, Clone)]
		pub struct MockFulfill {
			pub public_values: Vec<u8>,
		}
		impl Encode for MockFulfill {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.public_values.encode());
			}
		}
		impl Decode for MockFulfill {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let public_values = Decode::decode(input)?;
				Ok(Self { public_values })
			}
		}
		impl HasHeader for MockFulfill {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 17);
		}

		#[derive(Debug, Clone)]
		pub struct FailedSendMessageTxs {
			pub failed_txs: Vec<u32>,
		}
		impl Encode for FailedSendMessageTxs {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let failed_txs: Vec<Compact<u32>> = self.failed_txs.iter().map(|x| x.into()).collect();
				failed_txs.encode_to(dest);
			}
		}
		impl Decode for FailedSendMessageTxs {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let failed_txs = Vec::<Compact<u32>>::decode(input)?;
				let failed_txs = failed_txs.into_iter().map(|x| x.into()).collect();
				Ok(Self { failed_txs })
			}
		}
		impl HasHeader for FailedSendMessageTxs {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 11);
		}
	}
}

pub mod system {
	use super::*;
	pub const PALLET_ID: u8 = 0;

	pub mod types {
		use crate::types::substrate::{DispatchClass, Weight};

		use super::*;
		#[derive(Debug, Clone, Default, DecodeAsType, EncodeAsType)]
		pub struct AccountInfo {
			pub nonce: u32,
			pub consumers: u32,
			pub providers: u32,
			pub sufficients: u32,
			pub data: super::balances::types::AccountData,
		}
		impl Encode for AccountInfo {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.nonce.encode_to(dest);
				self.consumers.encode_to(dest);
				self.providers.encode_to(dest);
				self.sufficients.encode_to(dest);
				self.data.encode_to(dest);
			}
		}
		impl Decode for AccountInfo {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let nonce = Decode::decode(input)?;
				let consumers = Decode::decode(input)?;
				let providers = Decode::decode(input)?;
				let sufficients = Decode::decode(input)?;
				let data = Decode::decode(input)?;
				Ok(Self { nonce, consumers, providers, sufficients, data })
			}
		}

		#[derive(Debug, Clone)]
		pub struct DispatchInfo {
			/// Weight of this transaction.
			pub weight: Weight,
			/// Class of this transaction.
			pub class: DispatchClass,
			/// Does this transaction pay fees.
			pub pays_fee: Pays,
			/// Does this transaction have custom fees.
			pub fee_modifier: DispatchFeeModifier,
		}
		impl Encode for DispatchInfo {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.weight.encode_to(dest);
				self.class.encode_to(dest);
				self.pays_fee.encode_to(dest);
				self.fee_modifier.encode_to(dest);
			}
		}
		impl Decode for DispatchInfo {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let weight = Decode::decode(input)?;
				let class = Decode::decode(input)?;
				let pays_fee = Decode::decode(input)?;
				let fee_modifier = Decode::decode(input)?;
				Ok(Self { weight, class, pays_fee, fee_modifier })
			}
		}

		#[derive(Debug, Clone, Copy)]
		#[repr(u8)]
		pub enum Pays {
			/// Transactor will pay related fees.
			Yes = 0,
			/// Transactor will NOT pay related fees.
			No = 1,
		}
		impl Encode for Pays {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = *self as u8;
				variant.encode_to(dest);
			}
		}
		impl Decode for Pays {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::Yes),
					1 => Ok(Self::No),
					_ => Err("Failed to decode Pays. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone)]
		pub struct DispatchFeeModifier {
			pub weight_maximum_fee: Option<u128>,
			pub weight_fee_divider: Option<u32>,
			pub weight_fee_multiplier: Option<u32>,
		}
		impl Encode for DispatchFeeModifier {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.weight_maximum_fee.encode_to(dest);
				self.weight_fee_divider.encode_to(dest);
				self.weight_fee_multiplier.encode_to(dest);
			}
		}
		impl Decode for DispatchFeeModifier {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let weight_maximum_fee = Decode::decode(input)?;
				let weight_fee_divider = Decode::decode(input)?;
				let weight_fee_multiplier = Decode::decode(input)?;
				Ok(Self {
					weight_maximum_fee,
					weight_fee_divider,
					weight_fee_multiplier,
				})
			}
		}

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum DispatchError {
			Other = 0,
			/// Failed to lookup some data.
			CannotLookup = 1,
			/// A bad origin.
			BadOrigin = 2,
			/// A custom error in a module.
			Module(ModuleError) = 3,
			/// At least one consumer is remaining so the account cannot be destroyed.
			ConsumerRemaining = 4,
			/// There are no providers so the account cannot be created.
			NoProviders = 5,
			/// There are too many consumers so the account cannot be created.
			TooManyConsumers = 6,
			/// An error to do with tokens.
			Token(TokenError) = 7,
			/// An arithmetic error.
			Arithmetic(ArithmeticError) = 8,
			/// The number of transactional layers has been reached, or we are not in a transactional layer.
			Transactional(TransactionalError) = 9,
			/// Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
			Exhausted = 10,
			/// The state is corrupt; this is generally not going to fix itself.
			Corruption = 11,
			/// Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
			Unavailable = 12,
			/// Root origin is not allowed.
			RootNotAllowed = 13,
		}
		impl Encode for DispatchError {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::Module(x) => x.encode_to(dest),
					Self::Token(x) => x.encode_to(dest),
					Self::Arithmetic(x) => x.encode_to(dest),
					Self::Transactional(x) => x.encode_to(dest),
					_ => (),
				}
			}
		}
		impl Decode for DispatchError {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::Other),
					1 => Ok(Self::CannotLookup),
					2 => Ok(Self::BadOrigin),
					3 => Ok(Self::Module(ModuleError::decode(input)?)),
					4 => Ok(Self::ConsumerRemaining),
					5 => Ok(Self::NoProviders),
					6 => Ok(Self::TooManyConsumers),
					7 => Ok(Self::Token(TokenError::decode(input)?)),
					8 => Ok(Self::Arithmetic(ArithmeticError::decode(input)?)),
					9 => Ok(Self::Transactional(TransactionalError::decode(input)?)),
					10 => Ok(Self::Exhausted),
					11 => Ok(Self::Corruption),
					12 => Ok(Self::Unavailable),
					13 => Ok(Self::RootNotAllowed),
					_ => Err("Failed to decode Runtime Call. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone)]
		pub struct ModuleError {
			/// Module index, matching the metadata module index.
			pub index: u8,
			/// Module specific error value.
			pub error: [u8; 4],
		}
		impl Encode for ModuleError {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.index.encode_to(dest);
				self.error.encode_to(dest);
			}
		}
		impl Decode for ModuleError {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let index = Decode::decode(input)?;
				let error = Decode::decode(input)?;
				Ok(Self { index, error })
			}
		}

		#[derive(Debug, Clone, Copy)]
		#[repr(u8)]
		pub enum TokenError {
			/// Funds are unavailable.
			FundsUnavailable = 0,
			/// Some part of the balance gives the only provider reference to the account and thus cannot
			/// be (re)moved.
			OnlyProvider = 1,
			/// Account cannot exist with the funds that would be given.
			BelowMinimum = 2,
			/// Account cannot be created.
			CannotCreate = 3,
			/// The asset in question is unknown.
			UnknownAsset = 4,
			/// Funds exist but are frozen.
			Frozen = 5,
			/// Operation is not supported by the asset.
			Unsupported = 6,
			/// Account cannot be created for a held balance.
			CannotCreateHold = 7,
			/// Withdrawal would cause unwanted loss of account.
			NotExpendable = 8,
			/// Account cannot receive the assets.
			Blocked = 9,
		}
		impl Encode for TokenError {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = *self as u8;
				variant.encode_to(dest);
			}
		}
		impl Decode for TokenError {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::FundsUnavailable),
					1 => Ok(Self::OnlyProvider),
					2 => Ok(Self::BelowMinimum),
					3 => Ok(Self::CannotCreate),
					4 => Ok(Self::UnknownAsset),
					5 => Ok(Self::Frozen),
					6 => Ok(Self::Unsupported),
					7 => Ok(Self::CannotCreateHold),
					8 => Ok(Self::NotExpendable),
					9 => Ok(Self::Blocked),
					_ => Err("Failed to decode TokenError. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone, Copy)]
		#[repr(u8)]
		pub enum ArithmeticError {
			/// Underflow.
			Underflow = 0,
			/// Overflow.
			Overflow = 1,
			/// Division by zero.
			DivisionByZero = 2,
		}
		impl Encode for ArithmeticError {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = *self as u8;
				variant.encode_to(dest);
			}
		}
		impl Decode for ArithmeticError {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::Underflow),
					1 => Ok(Self::Overflow),
					2 => Ok(Self::DivisionByZero),
					_ => Err("Failed to decode ArithmeticError. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone, Copy)]
		#[repr(u8)]
		pub enum TransactionalError {
			LimitReached = 0,
			NoLayer = 1,
		}
		impl Encode for TransactionalError {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = *self as u8;
				variant.encode_to(dest);
			}
		}
		impl Decode for TransactionalError {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::LimitReached),
					1 => Ok(Self::NoLayer),
					_ => Err("Failed to decode TransactionalError. Unknown variant".into()),
				}
			}
		}
	}

	pub mod storage {
		use super::{system::types::AccountInfo, *};

		pub struct Account;
		impl StorageMap for Account {
			type KEY = AccountId;
			type VALUE = AccountInfo;

			const KEY_HASHER: StorageHasher = StorageHasher::Blake2_128Concat;
			const PALLET_NAME: &str = "System";
			const STORAGE_NAME: &str = "Account";
		}
	}

	pub mod events {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct ExtrinsicSuccess {
			pub dispatch_info: super::types::DispatchInfo,
		}
		impl HasHeader for ExtrinsicSuccess {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}
		impl Decode for ExtrinsicSuccess {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let dispatch_info = Decode::decode(input)?;
				Ok(Self { dispatch_info })
			}
		}

		#[derive(Debug, Clone)]
		pub struct ExtrinsicFailed {
			pub dispatch_error: super::types::DispatchError,
			pub dispatch_info: super::types::DispatchInfo,
		}
		impl HasHeader for ExtrinsicFailed {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
		impl Decode for ExtrinsicFailed {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let dispatch_error = Decode::decode(input)?;
				let dispatch_info = Decode::decode(input)?;
				Ok(Self { dispatch_error, dispatch_info })
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct Remark {
			pub remark: Vec<u8>,
		}
		impl Encode for Remark {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.remark.encode_to(dest);
			}
		}
		impl Decode for Remark {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let remark = Vec::<u8>::decode(input)?;
				Ok(Self { remark })
			}
		}
		impl HasHeader for Remark {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct SetCode {
			pub code: Vec<u8>,
		}
		impl Encode for SetCode {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.code.encode_to(dest);
			}
		}
		impl Decode for SetCode {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let code = Vec::<u8>::decode(input)?;
				Ok(Self { code })
			}
		}
		impl HasHeader for SetCode {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Clone)]
		pub struct SetCodeWithoutChecks {
			pub code: Vec<u8>,
		}
		impl Encode for SetCodeWithoutChecks {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.code.encode_to(dest);
			}
		}
		impl Decode for SetCodeWithoutChecks {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let code = Vec::<u8>::decode(input)?;
				Ok(Self { code })
			}
		}
		impl HasHeader for SetCodeWithoutChecks {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Debug, Clone)]
		pub struct RemarkWithEvent {
			pub remark: Vec<u8>,
		}
		impl Encode for RemarkWithEvent {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.remark.encode_to(dest);
			}
		}
		impl Decode for RemarkWithEvent {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let remark = Vec::<u8>::decode(input)?;
				Ok(Self { remark })
			}
		}
		impl HasHeader for RemarkWithEvent {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 7);
		}
	}
}

pub mod timestamp {
	use super::*;
	pub const PALLET_ID: u8 = 3;

	pub mod storage {
		use super::*;

		pub struct Now;
		impl StorageValue for Now {
			type VALUE = u64;

			const PALLET_NAME: &str = "Timestamp";
			const STORAGE_NAME: &str = "Now";
		}

		pub struct DidUpdate;
		impl StorageValue for DidUpdate {
			type VALUE = bool;

			const PALLET_NAME: &str = "Timestamp";
			const STORAGE_NAME: &str = "DidUpdate";
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct Set {
			pub now: u64,
		}
		impl Encode for Set {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.now).encode_to(dest);
			}
		}
		impl Decode for Set {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let now = Compact::<u64>::decode(input)?.0;
				Ok(Self { now })
			}
		}
		impl HasHeader for Set {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}
	}
}

pub mod staking {
	use super::*;
	pub const PALLET_ID: u8 = 10;

	pub mod types {
		use super::*;
		pub type SessionIndex = u32;

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum RewardDestination {
			Staked = 0,
			Stash = 1,
			Controller = 2,
			Account(AccountId) = 3,
			None = 4,
		}
		impl Encode for RewardDestination {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				match self {
					RewardDestination::Staked => 0u8.encode_to(dest),
					RewardDestination::Stash => 1u8.encode_to(dest),
					RewardDestination::Controller => 2u8.encode_to(dest),
					RewardDestination::Account(x) => {
						3u8.encode_to(dest);
						x.encode_to(dest);
					},
					RewardDestination::None => 4u8.encode_to(dest),
				}
			}
		}
		impl Decode for RewardDestination {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::Staked),
					1 => Ok(Self::Stash),
					2 => Ok(Self::Controller),
					3 => {
						let account_id = AccountId::decode(input)?;
						Ok(Self::Account(account_id))
					},
					4 => Ok(Self::None),
					_ => Err("Failed to decode RewardDestination. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone)]
		pub struct ValidatorPrefs {
			pub commission: u32, // Compact Perbill
			pub blocked: bool,
		}
		impl Encode for ValidatorPrefs {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.commission).encode_to(dest);
				self.blocked.encode_to(dest);
			}
		}
		impl Decode for ValidatorPrefs {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let commission = Compact::<u32>::decode(input)?.0;
				let blocked = Decode::decode(input)?;
				Ok(Self { commission, blocked })
			}
		}
	}

	pub mod events {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct Bonded {
			pub stash: AccountId,
			pub amount: u128,
		}
		impl Encode for Bonded {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.stash.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Bonded {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let stash = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { stash, amount })
			}
		}
		impl HasHeader for Bonded {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 6);
		}

		#[derive(Debug, Clone)]
		pub struct Unbonded {
			pub stash: AccountId,
			pub amount: u128,
		}
		impl Encode for Unbonded {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.stash.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Unbonded {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let stash = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { stash, amount })
			}
		}
		impl HasHeader for Unbonded {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 7);
		}

		#[derive(Debug, Clone)]
		pub struct ValidatorPrefsSet {
			pub stash: AccountId,
			pub prefs: super::types::ValidatorPrefs,
		}
		impl Encode for ValidatorPrefsSet {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.stash.encode_to(dest);
				self.prefs.encode_to(dest);
			}
		}
		impl Decode for ValidatorPrefsSet {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let stash = Decode::decode(input)?;
				let prefs = Decode::decode(input)?;
				Ok(Self { stash, prefs })
			}
		}
		impl HasHeader for ValidatorPrefsSet {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 13);
		}

		#[derive(Debug, Clone)]
		pub struct Chilled {
			pub stash: AccountId,
		}
		impl Encode for Chilled {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.stash.encode_to(dest);
			}
		}
		impl Decode for Chilled {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let stash = Decode::decode(input)?;
				Ok(Self { stash })
			}
		}
		impl HasHeader for Chilled {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 11);
		}

		// TODO tests
		#[derive(Debug, Clone)]
		pub struct EraPaid {
			pub era_index: u32,
			pub validator_payout: u128,
			pub remainder: u128,
		}
		impl Encode for EraPaid {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.era_index.encode_to(dest);
				self.validator_payout.encode_to(dest);
				self.remainder.encode_to(dest);
			}
		}
		impl Decode for EraPaid {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let era_index = Decode::decode(input)?;
				let validator_payout = Decode::decode(input)?;
				let remainder = Decode::decode(input)?;
				Ok(Self { era_index, validator_payout, remainder })
			}
		}
		impl HasHeader for EraPaid {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct Rewarded {
			pub stash: AccountId,
			pub dest: super::types::RewardDestination,
			pub amount: u128,
		}
		impl Encode for Rewarded {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.stash.encode_to(dest);
				self.dest.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Rewarded {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let stash = Decode::decode(input)?;
				let dest = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { stash, dest, amount })
			}
		}
		impl HasHeader for Rewarded {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		// TODO tests
		#[derive(Debug, Clone)]
		pub struct Slashed {
			pub staker: AccountId,
			pub amount: u128,
		}
		impl Encode for Slashed {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.staker.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Slashed {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let staker = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { staker, amount })
			}
		}
		impl HasHeader for Slashed {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Clone)]
		pub struct Withdraw {
			pub stash: AccountId,
			pub amount: u128,
		}
		impl Encode for Withdraw {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.stash.encode_to(dest);
				self.amount.encode_to(dest);
			}
		}
		impl Decode for Withdraw {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let stash = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { stash, amount })
			}
		}
		impl HasHeader for Withdraw {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 8);
		}

		// TODO tests
		#[derive(Debug, Clone)]
		pub struct Kicked {
			pub nominator: AccountId,
			pub stash: AccountId,
		}
		impl Encode for Kicked {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.nominator.encode_to(dest);
				self.stash.encode_to(dest);
			}
		}
		impl Decode for Kicked {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let nominator = Decode::decode(input)?;
				let stash = Decode::decode(input)?;
				Ok(Self { nominator, stash })
			}
		}
		impl HasHeader for Kicked {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 9);
		}

		#[derive(Debug, Clone)]
		pub struct PayoutStarted {
			pub era_index: u32,
			pub validator_stash: AccountId,
		}
		impl Encode for PayoutStarted {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.era_index.encode_to(dest);
				self.validator_stash.encode_to(dest);
			}
		}
		impl Decode for PayoutStarted {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let era_index = Decode::decode(input)?;
				let validator_stash = Decode::decode(input)?;
				Ok(Self { era_index, validator_stash })
			}
		}
		impl HasHeader for PayoutStarted {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 12);
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		pub struct Bond {
			pub value: u128, // Compact
			pub payee: super::types::RewardDestination,
		}
		impl Encode for Bond {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.value).encode_to(dest);
				self.payee.encode_to(dest);
			}
		}
		impl Decode for Bond {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Compact::<u128>::decode(input)?.0;
				let payee = Decode::decode(input)?;
				Ok(Self { value, payee })
			}
		}
		impl HasHeader for Bond {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct BondExtra {
			pub value: u128, // Compact
		}
		impl Encode for BondExtra {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.value).encode_to(dest);
			}
		}
		impl Decode for BondExtra {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Compact::<u128>::decode(input)?.0;
				Ok(Self { value })
			}
		}
		impl HasHeader for BondExtra {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Debug, Clone)]
		pub struct Unbond {
			pub value: u128, // Compact
		}
		impl Encode for Unbond {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.value).encode_to(dest);
			}
		}
		impl Decode for Unbond {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Compact::<u128>::decode(input)?.0;
				Ok(Self { value })
			}
		}
		impl HasHeader for Unbond {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Clone)]
		pub struct Rebond {
			pub value: u128, // Compact
		}
		impl Encode for Rebond {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				Compact(self.value).encode_to(dest);
			}
		}
		impl Decode for Rebond {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let value = Compact::<u128>::decode(input)?.0;
				Ok(Self { value })
			}
		}
		impl HasHeader for Rebond {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 19);
		}

		#[derive(Debug, Clone)]
		pub struct Validate {
			pub prefs: super::types::ValidatorPrefs,
		}
		impl Encode for Validate {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.prefs.encode_to(dest);
			}
		}
		impl Decode for Validate {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let prefs = Decode::decode(input)?;
				Ok(Self { prefs })
			}
		}
		impl HasHeader for Validate {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 4);
		}

		#[derive(Debug, Clone)]
		pub struct Nominate {
			pub targets: Vec<MultiAddress>,
		}
		impl Encode for Nominate {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.targets.encode_to(dest);
			}
		}
		impl Decode for Nominate {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let targets = Decode::decode(input)?;
				Ok(Self { targets })
			}
		}
		impl HasHeader for Nominate {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 5);
		}

		// TODO tests
		#[derive(Debug, Clone)]
		pub struct ChillOther {
			pub stash: AccountId,
		}
		impl Encode for ChillOther {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.stash.encode_to(dest);
			}
		}
		impl Decode for ChillOther {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let stash = Decode::decode(input)?;
				Ok(Self { stash })
			}
		}
		impl HasHeader for ChillOther {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 23);
		}

		#[derive(Debug, Clone)]
		pub struct PayoutStakers {
			pub validator_stash: AccountId,
			pub era: u32,
		}
		impl Encode for PayoutStakers {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.validator_stash.encode_to(dest);
				self.era.encode_to(dest);
			}
		}
		impl Decode for PayoutStakers {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let validator_stash = Decode::decode(input)?;
				let era = Decode::decode(input)?;
				Ok(Self { validator_stash, era })
			}
		}
		impl HasHeader for PayoutStakers {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 18);
		}

		#[derive(Debug, Clone)]
		pub struct SetController {}
		impl Encode for SetController {
			fn encode_to<T: codec::Output + ?Sized>(&self, _dest: &mut T) {}
		}
		impl Decode for SetController {
			fn decode<I: codec::Input>(_input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self {})
			}
		}
		impl HasHeader for SetController {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 8);
		}

		#[derive(Debug, Clone)]
		pub struct SetPayee {
			pub payee: super::types::RewardDestination,
		}
		impl Encode for SetPayee {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.payee.encode_to(dest);
			}
		}
		impl Decode for SetPayee {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let payee = Decode::decode(input)?;
				Ok(Self { payee })
			}
		}
		impl HasHeader for SetPayee {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 7);
		}

		#[derive(Debug, Clone)]
		pub struct Chill {}
		impl Encode for Chill {
			fn encode_to<T: codec::Output + ?Sized>(&self, _dest: &mut T) {}
		}
		impl Decode for Chill {
			fn decode<I: codec::Input>(_input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self {})
			}
		}
		impl HasHeader for Chill {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 6);
		}

		#[derive(Debug, Clone)]
		pub struct WithdrawUnbonded {
			pub num_slashing_spans: u32,
		}
		impl Encode for WithdrawUnbonded {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.num_slashing_spans.encode_to(dest);
			}
		}
		impl Decode for WithdrawUnbonded {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let num_slashing_spans = Decode::decode(input)?;
				Ok(Self { num_slashing_spans })
			}
		}
		impl HasHeader for WithdrawUnbonded {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		// TODO tests
		#[derive(Debug, Clone)]
		pub struct ReapStash {
			pub stash: AccountId,
			pub num_slashing_spans: u32,
		}
		impl Encode for ReapStash {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.stash.encode_to(dest);
				self.num_slashing_spans.encode_to(dest);
			}
		}
		impl Decode for ReapStash {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let stash = Decode::decode(input)?;
				let num_slashing_spans = Decode::decode(input)?;
				Ok(Self { stash, num_slashing_spans })
			}
		}
		impl HasHeader for ReapStash {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 20);
		}

		#[derive(Debug, Clone)]
		pub struct Kick {
			pub who: Vec<MultiAddress>,
		}
		impl Encode for Kick {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.who.encode_to(dest);
			}
		}
		impl Decode for Kick {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let who = Decode::decode(input)?;
				Ok(Self { who })
			}
		}
		impl HasHeader for Kick {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 21);
		}

		// TODO tests
		#[derive(Debug, Clone)]
		pub struct ForceApplyMinCommission {
			pub validator_stash: AccountId,
		}
		impl Encode for ForceApplyMinCommission {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.validator_stash.encode_to(dest);
			}
		}
		impl Decode for ForceApplyMinCommission {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let validator_stash = Decode::decode(input)?;
				Ok(Self { validator_stash })
			}
		}
		impl HasHeader for ForceApplyMinCommission {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 24);
		}

		#[derive(Debug, Clone)]
		pub struct PayoutStakersByPage {
			pub validator_stash: AccountId,
			pub era: u32,
			pub page: u32,
		}
		impl Encode for PayoutStakersByPage {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.validator_stash.encode_to(dest);
				self.era.encode_to(dest);
				self.page.encode_to(dest);
			}
		}
		impl Decode for PayoutStakersByPage {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let validator_stash = Decode::decode(input)?;
				let era = Decode::decode(input)?;
				let page = Decode::decode(input)?;
				Ok(Self { validator_stash, era, page })
			}
		}
		impl HasHeader for PayoutStakersByPage {
			const HEADER_INDEX: (u8, u8) = (PALLET_ID, 26);
		}
	}
}

pub mod grandpa {
	use super::*;
	pub const PALLET_ID: u8 = 17;

	pub mod types {
		use super::*;
		pub type SetId = u64;

		#[derive(Debug, Clone)]
		pub struct StoredPendingChange {
			/// The block number this was scheduled at.
			pub scheduled_at: u32,
			/// The delay in blocks until it will be applied.
			pub delay: u32,
			/// The next authority set, weakly bounded in size by `Limit`.
			pub next_authorities: crate::grandpa::AuthorityList,
			/// If defined it means the change was forced and the given block number
			/// indicates the median last finalized block when the change was signaled.
			pub forced: Option<u32>,
		}
		impl Encode for StoredPendingChange {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.scheduled_at.encode_to(dest);
				self.delay.encode_to(dest);
				self.next_authorities.encode_to(dest);
				self.forced.encode_to(dest);
			}
		}
		impl Decode for StoredPendingChange {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let scheduled_at = Decode::decode(input)?;
				let delay = Decode::decode(input)?;
				let next_authorities = Decode::decode(input)?;
				let forced = Decode::decode(input)?;
				Ok(Self { scheduled_at, delay, next_authorities, forced })
			}
		}

		#[derive(Debug, Clone, codec::Decode, codec::Encode)]
		pub enum StoredState {
			/// The current authority set is live, and GRANDPA is enabled.
			Live,
			/// There is a pending pause event which will be enacted at the given block
			/// height.
			PendingPause {
				/// Block at which the intention to pause was scheduled.
				scheduled_at: u32,
				/// Number of blocks after which the change will be enacted.
				delay: u32,
			},
			/// The current GRANDPA authority set is paused.
			Paused,
			/// There is a pending resume event which will be enacted at the given block
			/// height.
			PendingResume {
				/// Block at which the intention to resume was scheduled.
				scheduled_at: u32,
				/// Number of blocks after which the change will be enacted.
				delay: u32,
			},
		}
	}

	pub mod storage {
		use super::*;
		use crate::avail::staking::types::SessionIndex;

		pub struct SetIdSession;
		impl StorageMap for SetIdSession {
			type KEY = types::SetId;
			type VALUE = SessionIndex;

			const KEY_HASHER: StorageHasher = StorageHasher::Twox64Concat;
			const PALLET_NAME: &str = "Grandpa";
			const STORAGE_NAME: &str = "SetIdSession";
		}

		pub struct CurrentSetId;
		impl StorageValue for CurrentSetId {
			type VALUE = types::SetId;

			const PALLET_NAME: &str = "Grandpa";
			const STORAGE_NAME: &str = "CurrentSetId";
		}

		pub struct Authorities;
		impl StorageValue for Authorities {
			type VALUE = crate::grandpa::AuthorityList;

			const PALLET_NAME: &str = "Grandpa";
			const STORAGE_NAME: &str = "Authorities";
		}

		pub struct PendingChange;
		impl StorageValue for PendingChange {
			type VALUE = types::StoredPendingChange;

			const PALLET_NAME: &str = "Grandpa";
			const STORAGE_NAME: &str = "PendingChange";
		}

		pub struct StoredState;
		impl StorageValue for StoredState {
			type VALUE = types::StoredState;

			const PALLET_NAME: &str = "Grandpa";
			const STORAGE_NAME: &str = "StoredState";
		}

		pub struct NextForced;
		impl StorageValue for NextForced {
			type VALUE = u32;

			const PALLET_NAME: &str = "Grandpa";
			const STORAGE_NAME: &str = "NextForced";
		}

		pub struct Stalled;
		impl StorageValue for Stalled {
			type VALUE = (u32, u32);

			const PALLET_NAME: &str = "Grandpa";
			const STORAGE_NAME: &str = "Stalled";
		}
	}
}
