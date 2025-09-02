use crate::{
	AccountId, H256, HasEventEmittedIndex, HasTxDispatchIndex, MultiAddress, StorageHasher, StorageMap, StorageValue,
	TransactionCall, transaction::AlreadyEncoded,
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
}
impl Decode for RuntimeCall {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let pallet_id = input.read_byte()?;
		let call_id = input.read_byte()?;

		if pallet_id == balances::PALLET_ID {
			if call_id == balances::tx::TransferAllowDeath::DISPATCH_INDEX.1 {
				let call = balances::tx::TransferAllowDeath::decode(input)?;
				return Ok(RuntimeCall::BalancesTransferAllDeath(call));
			}

			if call_id == balances::tx::TransferKeepAlive::DISPATCH_INDEX.1 {
				let call = balances::tx::TransferKeepAlive::decode(input)?;
				return Ok(RuntimeCall::BalancesTransferKeepAlive(call));
			}

			if call_id == balances::tx::TransferAll::DISPATCH_INDEX.1 {
				let call = balances::tx::TransferAll::decode(input)?;
				return Ok(RuntimeCall::BalancesTransferAll(call));
			}
		}

		if pallet_id == utility::PALLET_ID {
			if call_id == utility::tx::Batch::DISPATCH_INDEX.1 {
				let call = utility::tx::Batch::decode(input)?;
				return Ok(RuntimeCall::UtilityBatch(call));
			}

			if call_id == utility::tx::BatchAll::DISPATCH_INDEX.1 {
				let call = utility::tx::BatchAll::decode(input)?;
				return Ok(RuntimeCall::UtilityBatchAll(call));
			}

			if call_id == utility::tx::ForceBatch::DISPATCH_INDEX.1 {
				let call = utility::tx::ForceBatch::decode(input)?;
				return Ok(RuntimeCall::UtilityForceBatch(call));
			}
		}

		if pallet_id == system::PALLET_ID {
			if call_id == system::tx::Remark::DISPATCH_INDEX.1 {
				let call = system::tx::Remark::decode(input)?;
				return Ok(RuntimeCall::SystemRemark(call));
			}

			if call_id == system::tx::SetCode::DISPATCH_INDEX.1 {
				let call = system::tx::SetCode::decode(input)?;
				return Ok(RuntimeCall::SystemSetCode(call));
			}

			if call_id == system::tx::SetCodeWithoutChecks::DISPATCH_INDEX.1 {
				let call = system::tx::SetCodeWithoutChecks::decode(input)?;
				return Ok(RuntimeCall::SystemSetCodeWithoutChecks(call));
			}

			if call_id == system::tx::RemarkWithEvent::DISPATCH_INDEX.1 {
				let call = system::tx::RemarkWithEvent::decode(input)?;
				return Ok(RuntimeCall::SystemRemarkWithEvent(call));
			}
		}

		if pallet_id == proxy::PALLET_ID {
			if call_id == proxy::tx::Proxy::DISPATCH_INDEX.1 {
				let call = proxy::tx::Proxy::decode(input)?;
				return Ok(RuntimeCall::ProxyProxy(call));
			}

			if call_id == proxy::tx::AddProxy::DISPATCH_INDEX.1 {
				let call = proxy::tx::AddProxy::decode(input)?;
				return Ok(RuntimeCall::ProxyAddProxy(call));
			}

			if call_id == proxy::tx::CreatePure::DISPATCH_INDEX.1 {
				let call = proxy::tx::CreatePure::decode(input)?;
				return Ok(RuntimeCall::ProxyCreatePure(call));
			}

			if call_id == proxy::tx::KillPure::DISPATCH_INDEX.1 {
				let call = proxy::tx::KillPure::decode(input)?;
				return Ok(RuntimeCall::ProxyKillPure(call));
			}

			if call_id == proxy::tx::RemoveProxies::DISPATCH_INDEX.1 {
				let call = proxy::tx::RemoveProxies::decode(input)?;
				return Ok(RuntimeCall::ProxyRemoveProxies(call));
			}

			if call_id == proxy::tx::RemoveProxy::DISPATCH_INDEX.1 {
				let call = proxy::tx::RemoveProxy::decode(input)?;
				return Ok(RuntimeCall::ProxyRemoveProxy(call));
			}
		}

		if pallet_id == multisig::PALLET_ID {
			if call_id == multisig::tx::ApproveAsMulti::DISPATCH_INDEX.1 {
				let call = multisig::tx::ApproveAsMulti::decode(input)?;
				return Ok(RuntimeCall::MultisigApproveAsMulti(call));
			}

			if call_id == multisig::tx::AsMulti::DISPATCH_INDEX.1 {
				let call = multisig::tx::AsMulti::decode(input)?;
				return Ok(RuntimeCall::MultisigAsMulti(call));
			}

			if call_id == multisig::tx::AsMultiThreshold1::DISPATCH_INDEX.1 {
				let call = multisig::tx::AsMultiThreshold1::decode(input)?;
				return Ok(RuntimeCall::MultisigAsMultiThreshold1(call));
			}

			if call_id == multisig::tx::CancelAsMulti::DISPATCH_INDEX.1 {
				let call = multisig::tx::CancelAsMulti::decode(input)?;
				return Ok(RuntimeCall::MultisigCancelAsMulti(call));
			}
		}

		if pallet_id == data_availability::PALLET_ID {
			if call_id == data_availability::tx::CreateApplicationKey::DISPATCH_INDEX.1 {
				let call = data_availability::tx::CreateApplicationKey::decode(input)?;
				return Ok(RuntimeCall::DataAvailabilityCreateApplicationKey(call));
			}
		}

		Err(codec::Error::from("Failed to decode runtime call"))
	}
}

pub mod data_availability {
	use super::*;
	pub const PALLET_ID: u8 = 29;

	pub mod storage {
		use super::*;
		use crate::chain_types::system::types::DispatchFeeModifier;

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
		impl HasEventEmittedIndex for ApplicationKeyCreated {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 0);
		}
		impl Encode for ApplicationKeyCreated {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.key.encode_to(dest);
				self.owner.encode_to(dest);
				self.id.encode_to(dest);
			}
		}
		impl Decode for ApplicationKeyCreated {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let key = Decode::decode(input)?;
				let owner = Decode::decode(input)?;
				let id = Decode::decode(input)?;
				Ok(Self { key, owner, id })
			}
		}

		#[derive(Debug, Clone)]
		pub struct DataSubmitted {
			pub who: AccountId,
			pub data_hash: H256,
		}
		impl HasEventEmittedIndex for DataSubmitted {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 1);
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
		impl HasTxDispatchIndex for CreateApplicationKey {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
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
		impl HasTxDispatchIndex for SubmitData {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
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
		impl HasEventEmittedIndex for Endowed {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 0);
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
		impl HasEventEmittedIndex for DustLost {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 1);
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
		impl HasEventEmittedIndex for Transfer {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 2);
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
		impl HasEventEmittedIndex for Reserved {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 4);
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
		impl HasEventEmittedIndex for Unreserved {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 5);
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
		impl HasEventEmittedIndex for Deposit {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 7);
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
		impl HasEventEmittedIndex for Withdraw {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 8);
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
		impl HasEventEmittedIndex for Slashed {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 9);
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
		impl HasEventEmittedIndex for Locked {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 17);
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
		impl HasEventEmittedIndex for Unlocked {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 18);
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
		impl HasEventEmittedIndex for Frozen {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 19);
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
		impl HasEventEmittedIndex for Thawed {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 20);
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
		impl HasTxDispatchIndex for TransferAllowDeath {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
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
		impl HasTxDispatchIndex for TransferKeepAlive {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
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
		impl HasTxDispatchIndex for TransferAll {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
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
		impl HasEventEmittedIndex for BatchInterrupted {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 0);
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
		impl HasEventEmittedIndex for BatchCompleted {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
		impl Decode for BatchCompleted {
			fn decode<I: codec::Input>(_input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self)
			}
		}

		/// Batch of dispatches completed but has error
		#[derive(Debug, Clone)]
		pub struct BatchCompletedWithErrors;
		impl HasEventEmittedIndex for BatchCompletedWithErrors {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 2);
		}
		impl Decode for BatchCompletedWithErrors {
			fn decode<I: codec::Input>(_input: &mut I) -> Result<Self, codec::Error> {
				Ok(Self)
			}
		}

		/// A single item within a Batch of dispatches has completed with no error
		#[derive(Debug, Clone)]
		pub struct ItemCompleted;
		impl HasEventEmittedIndex for ItemCompleted {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 3);
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
		impl HasEventEmittedIndex for ItemFailed {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 4);
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
		impl HasEventEmittedIndex for DispatchedAs {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 5);
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

			pub fn add_calls(&mut self, value: Vec<TransactionCall>) {
				for v in value {
					self.add_call(v);
				}
			}

			pub fn add_call(&mut self, value: TransactionCall) {
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
				let calls = AlreadyEncoded::decode(input)?.0;
				Ok(Self { length, calls })
			}
		}
		impl HasTxDispatchIndex for Batch {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
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

			pub fn add_calls(&mut self, value: Vec<TransactionCall>) {
				for v in value {
					self.add_call(v);
				}
			}

			pub fn add_call(&mut self, value: TransactionCall) {
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
				let calls = AlreadyEncoded::decode(input)?.0;
				Ok(Self { length, calls })
			}
		}
		impl HasTxDispatchIndex for BatchAll {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
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

			pub fn add_calls(&mut self, value: Vec<TransactionCall>) {
				for v in value {
					self.add_call(v);
				}
			}

			pub fn add_call(&mut self, value: TransactionCall) {
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
				let calls = AlreadyEncoded::decode(input)?.0;
				Ok(Self { length, calls })
			}
		}
		impl HasTxDispatchIndex for ForceBatch {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
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
		impl HasEventEmittedIndex for ProxyExecuted {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 0);
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
		impl HasEventEmittedIndex for PureCreated {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 1);
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
		impl HasEventEmittedIndex for Announced {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 2);
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
		impl HasEventEmittedIndex for ProxyAdded {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 3);
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
		impl HasEventEmittedIndex for ProxyRemoved {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 4);
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
			pub call: TransactionCall,
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
		impl HasTxDispatchIndex for Proxy {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
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
		impl HasTxDispatchIndex for AddProxy {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
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
		impl HasTxDispatchIndex for RemoveProxy {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
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
		impl HasTxDispatchIndex for RemoveProxies {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
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
		impl HasTxDispatchIndex for CreatePure {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
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
		impl HasTxDispatchIndex for KillPure {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 5);
		}
	}
}

pub mod multisig {
	use super::*;
	pub const PALLET_ID: u8 = 34;

	pub mod types {
		use super::*;
		pub use crate::from_substrate::Weight;

		#[derive(Debug, Clone, Copy)]
		pub struct Timepoint {
			height: u32,
			index: u32,
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
		impl HasEventEmittedIndex for NewMultisig {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 0);
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
		impl HasEventEmittedIndex for MultisigApproval {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 1);
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
		impl HasEventEmittedIndex for MultisigExecuted {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 2);
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
		impl HasEventEmittedIndex for MultisigCancelled {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 3);
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
			pub call: TransactionCall,
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
		impl HasTxDispatchIndex for AsMultiThreshold1 {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct AsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub maybe_timepoint: Option<super::types::Timepoint>,
			pub call: TransactionCall,
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
				let call = Decode::decode(input)?;
				let max_weight = Decode::decode(input)?;
				Ok(Self {
					threshold,
					other_signatories,
					maybe_timepoint,
					call,
					max_weight,
				})
			}
		}
		impl HasTxDispatchIndex for AsMulti {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
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
		impl HasTxDispatchIndex for ApproveAsMulti {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
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
		impl HasTxDispatchIndex for CancelAsMulti {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}
	}
}

pub mod vector {
	use super::*;
	pub const PALLET_ID: u8 = 39;

	pub mod types {
		use super::*;
		pub use crate::from_substrate::Weight;

		/// Message type used to bridge between Avail & other chains
		#[derive(Debug, Clone, Serialize, Deserialize)]
		#[serde(rename_all = "camelCase")]
		pub struct AddressedMessage {
			pub message: Message,
			pub from: H256,
			pub to: H256,
			pub origin_domain: Compact<u32>,
			pub destination_domain: Compact<u32>,
			/// Unique identifier for the message
			pub id: Compact<u64>,
		}
		impl Encode for AddressedMessage {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.message.encode_to(dest);
				self.from.encode_to(dest);
				self.to.encode_to(dest);
				self.origin_domain.encode_to(dest);
				self.destination_domain.encode_to(dest);
				self.id.encode_to(dest);
			}
		}
		impl Decode for AddressedMessage {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let message = Decode::decode(input)?;
				let from = Decode::decode(input)?;
				let to = Decode::decode(input)?;
				let origin_domain = Decode::decode(input)?;
				let destination_domain = Decode::decode(input)?;
				let id = Decode::decode(input)?;
				Ok(Self { message, from, to, origin_domain, destination_domain, id })
			}
		}

		/// Possible types of Messages allowed by Avail to bridge to other chains.
		#[derive(Debug, Clone, Serialize, Deserialize)]
		#[repr(u8)]
		pub enum Message {
			ArbitraryMessage(Vec<u8>) = 0,
			FungibleToken { asset_id: H256, amount: Compact<u128> } = 1,
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
						amount.encode_to(dest);
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
						let amount = Decode::decode(input)?;
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
			pub slot: Compact<u64>,
		}
		impl Encode for FulfillCall {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.function_id.encode_to(dest);
				self.input.encode_to(dest);
				self.output.encode_to(dest);
				self.proof.encode_to(dest);
				self.slot.encode_to(dest);
			}
		}
		impl Decode for FulfillCall {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let function_id = Decode::decode(input)?;
				let inputt = Decode::decode(input)?;
				let output = Decode::decode(input)?;
				let proof = Decode::decode(input)?;
				let slot = Decode::decode(input)?;
				Ok(Self { function_id, input: inputt, output, proof, slot })
			}
		}
		impl HasTxDispatchIndex for FulfillCall {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct Execute {
			pub slot: Compact<u64>,
			pub addr_message: super::types::AddressedMessage,
			pub account_proof: Vec<Vec<u8>>,
			pub storage_proof: Vec<Vec<u8>>,
		}
		impl Encode for Execute {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.slot.encode_to(dest);
				self.addr_message.encode_to(dest);
				self.account_proof.encode_to(dest);
				self.storage_proof.encode_to(dest);
			}
		}
		impl Decode for Execute {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let slot = Decode::decode(input)?;
				let addr_message = Decode::decode(input)?;
				let account_proof = Decode::decode(input)?;
				let storage_proof = Decode::decode(input)?;
				Ok(Self { slot, addr_message, account_proof, storage_proof })
			}
		}
		impl HasTxDispatchIndex for Execute {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Debug, Clone)]
		pub struct SourceChainFroze {
			pub source_chain_id: Compact<u32>,
			pub frozen: bool,
		}
		impl Encode for SourceChainFroze {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.source_chain_id.encode_to(dest);
				self.frozen.encode_to(dest);
			}
		}
		impl Decode for SourceChainFroze {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let source_chain_id = Decode::decode(input)?;
				let frozen = Decode::decode(input)?;
				Ok(Self { source_chain_id, frozen })
			}
		}
		impl HasTxDispatchIndex for SourceChainFroze {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Clone)]
		pub struct SendMessage {
			pub slot: Compact<u64>,
			pub message: super::types::Message,
			pub to: H256,
			pub domain: Compact<u32>,
		}
		impl Encode for SendMessage {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				self.slot.encode_to(dest);
				self.message.encode_to(dest);
				self.to.encode_to(dest);
				self.domain.encode_to(dest);
			}
		}
		impl Decode for SendMessage {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let slot = Decode::decode(input)?;
				let message = Decode::decode(input)?;
				let to = Decode::decode(input)?;
				let domain = Decode::decode(input)?;
				Ok(Self { slot, message, to, domain })
			}
		}
		impl HasTxDispatchIndex for SendMessage {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
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
		impl HasTxDispatchIndex for SetPoseidonHash {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
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
		impl HasTxDispatchIndex for SetBroadcaster {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 5);
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
		impl HasTxDispatchIndex for SetWhitelistedDomains {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 6);
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
		impl HasTxDispatchIndex for SetConfiguration {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 7);
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
		impl HasTxDispatchIndex for SetFunctionIds {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 8);
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
		impl HasTxDispatchIndex for SetStepVerificationKey {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 9);
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
		impl HasTxDispatchIndex for SetRotateVerificationKey {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 10);
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
		impl HasTxDispatchIndex for SetUpdater {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 12);
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
		impl HasTxDispatchIndex for Fulfill {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 13);
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
		impl HasTxDispatchIndex for SetSp1VerificationKey {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 14);
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
		impl HasTxDispatchIndex for SetSyncCommitteeHash {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 15);
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
		impl HasTxDispatchIndex for EnableMock {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 16);
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
		impl HasTxDispatchIndex for MockFulfill {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 17);
		}
	}
}

pub mod system {
	use super::*;
	pub const PALLET_ID: u8 = 0;

	pub mod types {
		use crate::from_substrate::{DispatchClass, Weight};

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
		use super::*;
		use crate::chain_types::system::types::AccountInfo;

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
		impl HasEventEmittedIndex for ExtrinsicSuccess {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 0);
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
		impl HasEventEmittedIndex for ExtrinsicFailed {
			const EMITTED_INDEX: (u8, u8) = (PALLET_ID, 1);
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
		impl HasTxDispatchIndex for Remark {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
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
		impl HasTxDispatchIndex for SetCode {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
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
		impl HasTxDispatchIndex for SetCodeWithoutChecks {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
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
		impl HasTxDispatchIndex for RemarkWithEvent {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 7);
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
		impl HasTxDispatchIndex for Set {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}
	}
}

pub mod staking {
	pub const PALLET_ID: u8 = 10;

	pub mod types {
		pub type SessionIndex = u32;
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
