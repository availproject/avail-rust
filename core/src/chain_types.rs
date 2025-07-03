use crate::{AccountId, MultiAddress, TransactionCall};
use codec::{Compact, Decode, Encode};
use primitive_types::H256;
use scale_decode::DecodeAsType;
use scale_encode::EncodeAsType;
use subxt_core::{
	storage::address::{StaticAddress, StaticStorageKey},
	utils::Yes,
};

pub trait TxEventEmittedIndex {
	// Pallet ID, Variant ID
	const EMITTED_INDEX: (u8, u8);
}

pub trait TransactionEventLike {
	fn from_raw(raw: &[u8]) -> Option<Box<Self>>;
}

impl<T: TxEventEmittedIndex + Encode + Decode> TransactionEventLike for T {
	fn from_raw(raw: &[u8]) -> Option<Box<T>> {
		if raw.len() < 2 {
			return None;
		}

		let (pallet_id, variant_id) = (raw[0], raw[1]);
		if Self::EMITTED_INDEX.0 != pallet_id || Self::EMITTED_INDEX.1 != variant_id {
			return None;
		}

		Self::decode(&mut &raw[2..]).ok().map(Box::new)
	}
}

pub trait TxDispatchIndex {
	// Pallet ID, Call ID
	const DISPATCH_INDEX: (u8, u8);
}

pub trait EventEmittedIndex {
	// Pallet ID, Event ID
	const EMITTED_INDEX: (u8, u8);
}

pub trait TransactionCallLike {
	fn to_call(&self) -> TransactionCall;
	fn from_ext(raw: &[u8]) -> Option<Box<Self>>;
}

impl<T: TxDispatchIndex + Encode + Decode> TransactionCallLike for T {
	fn to_call(&self) -> TransactionCall {
		TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, self.encode())
	}

	fn from_ext(raw_ext: &[u8]) -> Option<Box<T>> {
		if raw_ext.len() < 2 {
			return None;
		}

		let (pallet_id, call_id) = (raw_ext[0], raw_ext[1]);
		if Self::DISPATCH_INDEX.0 != pallet_id || Self::DISPATCH_INDEX.1 != call_id {
			return None;
		}

		Self::decode(&mut &raw_ext[2..]).ok().map(Box::new)
	}
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum RuntimeCall {
	DataAvailability(data_availability::tx::Call) = data_availability::PALLET_ID,
	Balances(balances::tx::Call) = balances::PALLET_ID,
	Utility(utility::tx::Call) = utility::PALLET_ID,
	Proxy(proxy::tx::Call) = proxy::PALLET_ID,
	Multisig(multisig::tx::Call) = multisig::PALLET_ID,
	System(system::tx::Call) = system::PALLET_ID,
}
impl RuntimeCall {
	pub fn pallet_index(&self) -> u8 {
		unsafe { *(self as *const _ as *const u8) }
	}

	pub fn call_index(&self) -> u8 {
		match self {
			Self::DataAvailability(call) => unsafe { *(call as *const _ as *const u8) },
			Self::Balances(call) => unsafe { *(call as *const _ as *const u8) },
			Self::Utility(call) => unsafe { *(call as *const _ as *const u8) },
			Self::Proxy(call) => unsafe { *(call as *const _ as *const u8) },
			Self::Multisig(call) => unsafe { *(call as *const _ as *const u8) },
			Self::System(call) => unsafe { *(call as *const _ as *const u8) },
		}
	}
}

impl Encode for RuntimeCall {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
		variant.encode_to(dest);
		match self {
			Self::DataAvailability(x) => x.encode_to(dest),
			Self::Balances(x) => x.encode_to(dest),
			Self::Utility(x) => x.encode_to(dest),
			Self::Proxy(x) => x.encode_to(dest),
			Self::Multisig(x) => x.encode_to(dest),
			Self::System(x) => x.encode_to(dest),
		}
	}
}

impl Decode for RuntimeCall {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let variant = u8::decode(input)?;
		match variant {
			data_availability::PALLET_ID => Ok(Self::DataAvailability(Decode::decode(input)?)),
			balances::PALLET_ID => Ok(Self::Balances(Decode::decode(input)?)),
			utility::PALLET_ID => Ok(Self::Utility(Decode::decode(input)?)),
			proxy::PALLET_ID => Ok(Self::Proxy(Decode::decode(input)?)),
			multisig::PALLET_ID => Ok(Self::Multisig(Decode::decode(input)?)),
			system::PALLET_ID => Ok(Self::System(Decode::decode(input)?)),
			_ => Err("Failed to decode Runtime Call. Unknown variant".into()),
		}
	}
}

impl TryFrom<&[u8]> for RuntimeCall {
	type Error = codec::Error;

	fn try_from(mut value: &[u8]) -> Result<Self, Self::Error> {
		Self::decode(&mut value)
	}
}

impl TryFrom<&Vec<u8>> for RuntimeCall {
	type Error = codec::Error;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<Vec<u8>> for RuntimeCall {
	type Error = codec::Error;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum RuntimeEvent {
	System(system::events::Event) = system::PALLET_ID,
	Utility(utility::events::Event) = utility::PALLET_ID,
	Balances(balances::events::Event) = balances::PALLET_ID,
	DataAvailability(data_availability::events::Event) = data_availability::PALLET_ID,
	Multisig(multisig::events::Event) = multisig::PALLET_ID,
	Proxy(multisig::events::Event) = proxy::PALLET_ID,
}
impl RuntimeEvent {
	pub fn pallet_index(&self) -> u8 {
		unsafe { *(self as *const _ as *const u8) }
	}

	pub fn variant_index(&self) -> u8 {
		match self {
			Self::System(call) => unsafe { *(call as *const _ as *const u8) },
			Self::Utility(call) => unsafe { *(call as *const _ as *const u8) },
			Self::Balances(call) => unsafe { *(call as *const _ as *const u8) },
			Self::DataAvailability(call) => unsafe { *(call as *const _ as *const u8) },
			Self::Multisig(call) => unsafe { *(call as *const _ as *const u8) },
			Self::Proxy(call) => unsafe { *(call as *const _ as *const u8) },
		}
	}
}
/* impl Encode for RuntimeEvent {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
		variant.encode_to(dest);
		match self {
			Self::System(x) => x.encode_to(dest),
			Self::Utility(x) => x.encode_to(dest),
			Self::DataAvailability(x) => x.encode_to(dest),
		}
	}
} */
impl Decode for RuntimeEvent {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		let variant = u8::decode(input)?;
		match variant {
			system::PALLET_ID => Ok(Self::System(Decode::decode(input)?)),
			utility::PALLET_ID => Ok(Self::Utility(Decode::decode(input)?)),
			balances::PALLET_ID => Ok(Self::Balances(Decode::decode(input)?)),
			data_availability::PALLET_ID => Ok(Self::DataAvailability(Decode::decode(input)?)),
			multisig::PALLET_ID => Ok(Self::Multisig(Decode::decode(input)?)),
			proxy::PALLET_ID => Ok(Self::Proxy(Decode::decode(input)?)),
			_ => Err("Failed to decode Runtime Event. Unknown variant".into()),
		}
	}
}
impl TryFrom<&[u8]> for RuntimeEvent {
	type Error = codec::Error;

	fn try_from(mut value: &[u8]) -> Result<Self, Self::Error> {
		Self::decode(&mut value)
	}
}

impl TryFrom<&Vec<u8>> for RuntimeEvent {
	type Error = codec::Error;

	fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

impl TryFrom<Vec<u8>> for RuntimeEvent {
	type Error = codec::Error;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(value.as_slice())
	}
}

pub mod data_availability {
	use super::*;
	pub const PALLET_ID: u8 = 29;

	pub mod events {
		use super::*;

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Event {
			ApplicationKeyCreated { key: Vec<u8>, owner: AccountId, id: u32 } = 0,
			DataSubmitted { who: AccountId, data_hash: H256 } = 1,
		}
		/* 		impl Encode for Event {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::ApplicationKeyCreated { key, owner, id } => {
						key.encode_to(dest);
						owner.encode_to(dest);
						Compact(*id).encode_to(dest);
					},
					Self::DataSubmitted { who, data_hash } => {
						who.encode_to(dest);
						data_hash.encode_to(dest);
					},
				}
			}
		} */
		impl Decode for Event {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => {
						let key = Decode::decode(input)?;
						let owner = Decode::decode(input)?;
						let id: Compact<u32> = Decode::decode(input)?;
						Ok(Self::ApplicationKeyCreated { key, owner, id: id.0 })
					},
					1 => Ok(Self::DataSubmitted {
						who: Decode::decode(input)?,
						data_hash: Decode::decode(input)?,
					}),
					_ => Err("Failed to decode DataAvailability Event. Unknown variant".into()),
				}
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Call {
			CreateApplicationKey(CreateApplicationKey) = CreateApplicationKey::DISPATCH_INDEX.1,
			SubmitData(SubmitData) = SubmitData::DISPATCH_INDEX.1,
		}
		impl Encode for Call {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::CreateApplicationKey(x) => x.encode_to(dest),
					Self::SubmitData(x) => x.encode_to(dest),
				}
			}
		}
		impl Decode for Call {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					val if val == CreateApplicationKey::DISPATCH_INDEX.1 => {
						Ok(Self::CreateApplicationKey(Decode::decode(input)?))
					},
					val if val == SubmitData::DISPATCH_INDEX.1 => Ok(Self::SubmitData(Decode::decode(input)?)),
					_ => Err("Failed to decode DataAvailability Call. Unknown variant".into()),
				}
			}
		}

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
		impl TxDispatchIndex for CreateApplicationKey {
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
		impl TxDispatchIndex for SubmitData {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
	}
}

pub mod balances {
	use super::*;
	pub const PALLET_ID: u8 = 6;

	pub mod types {
		use super::*;

		#[derive(Debug, Clone, DecodeAsType, EncodeAsType)]
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
				Ok(Self {
					free,
					reserved,
					frozen,
					flags,
				})
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

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Event {
			/// An account was created with some free balance.
			Endowed { account: AccountId, free_balance: u128 } = 0,
			/// An account was removed whose balance was non-zero but below ExistentialDeposit,
			/// resulting in an outright loss.
			DustLost { account: AccountId, amount: u128 } = 1,
			/// Transfer succeeded.
			Transfer {
				from: AccountId,
				to: AccountId,
				amount: u128,
			} = 2,
			/// A balance was set by root.
			BalanceSet { who: AccountId, free: u128 } = 3,
			/// Some balance was reserved (moved from free to reserved).
			Reserved { who: AccountId, amount: u128 } = 4,
			/// Some balance was unreserved (moved from reserved to free).
			Unreserved { who: AccountId, amount: u128 } = 5,
			/// Some balance was moved from the reserve of the first account to the second account.
			/// Final argument indicates the destination balance type.
			ReserveRepatriated {
				from: AccountId,
				to: AccountId,
				amount: u128,
				destination_status: super::types::BalanceStatus,
			} = 6,
			/// Some amount was deposited (e.g. for transaction fees).
			Deposit { who: AccountId, amount: u128 } = 7,
			/// Some amount was withdrawn from the account (e.g. for transaction fees).
			Withdraw { who: AccountId, amount: u128 } = 8,
			/// Some amount was removed from the account (e.g. for misbehavior).
			Slashed { who: AccountId, amount: u128 } = 9,
			/// Some amount was minted into an account.
			Minted { who: AccountId, amount: u128 } = 10,
			/// Some amount was burned from an account.
			Burned { who: AccountId, amount: u128 } = 11,
			/// Some amount was suspended from an account (it can be restored later).
			Suspended { who: AccountId, amount: u128 } = 12,
			/// Some amount was restored into an account.
			Restored { who: AccountId, amount: u128 } = 13,
			/// An account was upgraded.
			Upgraded { who: AccountId } = 14,
			/// Total issuance was increased by `amount`, creating a credit to be balanced.
			Issued { amount: u128 } = 15,
			/// Total issuance was decreased by `amount`, creating a debt to be balanced.
			Rescinded { amount: u128 } = 16,
			/// Some balance was locked.
			Locked { who: AccountId, amount: u128 } = 17,
			/// Some balance was unlocked.
			Unlocked { who: AccountId, amount: u128 } = 18,
			/// Some balance was frozen.
			Frozen { who: AccountId, amount: u128 } = 19,
			/// Some balance was thawed.
			Thawed { who: AccountId, amount: u128 } = 20,
			/// The `TotalIssuance` was forcefully changed.
			TotalIssuanceForced { old: u128, new: u128 } = 21,
		}
		/* 		impl Encode for Event {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::Endowed { account, free_balance } => {
						account.encode_to(dest);
						free_balance.encode_to(dest);
					},
					Self::DustLost { account, amount } => {
						account.encode_to(dest);
						amount.encode_to(dest);
					},
					Self::Transfer { from, to, amount } => {
						from.encode_to(dest);
						to.encode_to(dest);
						amount.encode_to(dest);
					},
					Self::BalanceSet { who, free } => {
						who.encode_to(dest);
						free.encode_to(dest);
					},
					Self::Reserved { who, amount } => {
						who.encode_to(dest);
						amount.encode_to(dest);
					},
					Self::Unreserved { who, amount } => {
						who.encode_to(dest);
						amount.encode_to(dest);
					},
					Self::ReserveRepatriated {
						from,
						to,
						amount,
						destination_status,
					} => {
						from.encode_to(dest);
						to.encode_to(dest);
						amount.encode_to(dest);
						destination_status.encode_to(dest);
					},
					Self::Deposit { who, amount } => {
						who.encode_to(dest);
						amount.encode_to(dest);
					},
					Self::Withdraw { who, amount } => {
						who.encode_to(dest);
						amount.encode_to(dest);
					},
					Self::Slashed { who, amount } => {
						who.encode_to(dest);
						amount.encode_to(dest);
					},
					Self::Minted { who, amount } => {
						who.encode_to(dest);
						amount.encode_to(dest);
					},
					_ => (),
				}
			}
		} */
		impl Decode for Event {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::Endowed {
						account: Decode::decode(input)?,
						free_balance: Decode::decode(input)?,
					}),
					1 => Ok(Self::DustLost {
						account: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					2 => Ok(Self::Transfer {
						from: Decode::decode(input)?,
						to: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					3 => Ok(Self::BalanceSet {
						who: Decode::decode(input)?,
						free: Decode::decode(input)?,
					}),
					4 => Ok(Self::Reserved {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					5 => Ok(Self::Unreserved {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					6 => Ok(Self::ReserveRepatriated {
						from: Decode::decode(input)?,
						to: Decode::decode(input)?,
						amount: Decode::decode(input)?,
						destination_status: Decode::decode(input)?,
					}),
					7 => Ok(Self::Deposit {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					8 => Ok(Self::Withdraw {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					9 => Ok(Self::Slashed {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					10 => Ok(Self::Minted {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					11 => Ok(Self::Burned {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					12 => Ok(Self::Suspended {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					13 => Ok(Self::Restored {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					14 => Ok(Self::Upgraded {
						who: Decode::decode(input)?,
					}),
					15 => Ok(Self::Issued {
						amount: Decode::decode(input)?,
					}),
					16 => Ok(Self::Rescinded {
						amount: Decode::decode(input)?,
					}),
					17 => Ok(Self::Locked {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					18 => Ok(Self::Unlocked {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					19 => Ok(Self::Frozen {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					20 => Ok(Self::Thawed {
						who: Decode::decode(input)?,
						amount: Decode::decode(input)?,
					}),
					21 => Ok(Self::TotalIssuanceForced {
						old: Decode::decode(input)?,
						new: Decode::decode(input)?,
					}),
					_ => Err("Failed to decode Balances Event. Unknown variant".into()),
				}
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Call {
			TransferAllowDeath(TransferAllowDeath) = TransferAllowDeath::DISPATCH_INDEX.1,
			TransferKeepAlive(TransferKeepAlive) = TransferKeepAlive::DISPATCH_INDEX.1,
			TransferAll(TransferAll) = TransferAll::DISPATCH_INDEX.1,
		}
		impl Encode for Call {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::TransferAllowDeath(x) => x.encode_to(dest),
					Self::TransferKeepAlive(x) => x.encode_to(dest),
					Self::TransferAll(x) => x.encode_to(dest),
				}
			}
		}
		impl Decode for Call {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					val if val == TransferAllowDeath::DISPATCH_INDEX.1 => {
						Ok(Self::TransferAllowDeath(Decode::decode(input)?))
					},
					val if val == TransferKeepAlive::DISPATCH_INDEX.1 => {
						Ok(Self::TransferKeepAlive(Decode::decode(input)?))
					},
					val if val == TransferAll::DISPATCH_INDEX.1 => Ok(Self::TransferAll(Decode::decode(input)?)),
					_ => Err("Failed to decode Balances Call. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone)]
		pub struct TransferAllowDeath {
			pub dest: MultiAddress,
			pub amount: Compact<u128>,
		}
		impl Encode for TransferAllowDeath {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.dest.encode());
				dest.write(&self.amount.encode());
			}
		}
		impl Decode for TransferAllowDeath {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let dest = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { dest, amount })
			}
		}
		impl TxDispatchIndex for TransferAllowDeath {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct TransferKeepAlive {
			pub dest: MultiAddress,
			pub amount: Compact<u128>,
		}
		impl Encode for TransferKeepAlive {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.dest.encode());
				dest.write(&self.amount.encode());
			}
		}
		impl Decode for TransferKeepAlive {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let dest = Decode::decode(input)?;
				let amount = Decode::decode(input)?;
				Ok(Self { dest, amount })
			}
		}
		impl TxDispatchIndex for TransferKeepAlive {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Debug, Clone)]
		pub struct TransferAll {
			pub dest: MultiAddress,
			pub keep_alive: bool,
		}
		impl Encode for TransferAll {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.dest.encode());
				dest.write(&self.keep_alive.encode());
			}
		}
		impl Decode for TransferAll {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let dest = Decode::decode(input)?;
				let keep_alive = Decode::decode(input)?;
				Ok(Self { dest, keep_alive })
			}
		}
		impl TxDispatchIndex for TransferAll {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
	}
}

pub mod utility {
	use super::*;
	pub const PALLET_ID: u8 = 1;

	pub mod events {
		use super::*;

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Event {
			/// Batch of dispatches did not complete fully. Index of first failing dispatch given, as
			/// well as the error.
			BatchInterrupted {
				index: u32,
				error: super::system::types::DispatchError,
			} = 0,
			/// Batch of dispatches completed fully with no error.
			BatchCompleted = 1,
			/// Batch of dispatches completed but has errors.
			BatchCompletedWithErrors = 2,
			/// A single item within a Batch of dispatches has completed with no error.
			ItemCompleted = 3,
			/// A single item within a Batch of dispatches has completed with error.
			ItemFailed { error: super::system::types::DispatchError } = 4,
			/// A call was dispatched.
			DispatchedAs {
				result: Result<(), super::system::types::DispatchError>,
			} = 5,
		}
		/* 		impl Encode for Event {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::BatchInterrupted { index, error } => {
						index.encode_to(dest);
						error.encode_to(dest);
					},
					Self::ItemFailed { error } => {
						error.encode_to(dest);
					},
					Self::DispatchedAs { result } => {
						result.encode_to(dest);
					},
					_ => (),
				}
			}
		} */
		impl Decode for Event {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::BatchInterrupted {
						index: Decode::decode(input)?,
						error: Decode::decode(input)?,
					}),
					1 => Ok(Self::BatchCompleted),
					2 => Ok(Self::BatchCompletedWithErrors),
					3 => Ok(Self::ItemCompleted),
					4 => Ok(Self::ItemFailed {
						error: Decode::decode(input)?,
					}),
					5 => Ok(Self::DispatchedAs {
						result: Decode::decode(input)?,
					}),
					_ => Err("Failed to decode System Event. Unknown variant".into()),
				}
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Call {
			Batch(Batch) = Batch::DISPATCH_INDEX.1,
			BatchAll(BatchAll) = BatchAll::DISPATCH_INDEX.1,
			ForceBatch(ForceBatch) = ForceBatch::DISPATCH_INDEX.1,
		}
		impl Encode for Call {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::Batch(x) => x.encode_to(dest),
					Self::BatchAll(x) => x.encode_to(dest),
					Self::ForceBatch(x) => x.encode_to(dest),
				}
			}
		}
		impl Decode for Call {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					val if val == Batch::DISPATCH_INDEX.1 => Ok(Self::Batch(Decode::decode(input)?)),
					val if val == BatchAll::DISPATCH_INDEX.1 => Ok(Self::BatchAll(Decode::decode(input)?)),
					val if val == ForceBatch::DISPATCH_INDEX.1 => Ok(Self::ForceBatch(Decode::decode(input)?)),
					_ => Err("Failed to decode Utility Call. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone)]
		pub struct Batch {
			pub calls: Vec<TransactionCall>,
		}
		impl Encode for Batch {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.calls.encode());
			}
		}
		impl Decode for Batch {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let calls = Decode::decode(input)?;
				Ok(Self { calls })
			}
		}
		impl TxDispatchIndex for Batch {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Debug, Clone)]
		pub struct BatchAll {
			pub calls: Vec<TransactionCall>,
		}
		impl Encode for BatchAll {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.calls.encode());
			}
		}
		impl Decode for BatchAll {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let calls = Decode::decode(input)?;
				Ok(Self { calls })
			}
		}
		impl TxDispatchIndex for BatchAll {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Clone)]
		pub struct ForceBatch {
			pub calls: Vec<TransactionCall>,
		}
		impl Encode for ForceBatch {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.calls.encode());
			}
		}
		impl Decode for ForceBatch {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let calls = Decode::decode(input)?;
				Ok(Self { calls })
			}
		}
		impl TxDispatchIndex for ForceBatch {
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

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Event {
			/// A proxy was executed correctly, with the given.
			ProxyExecuted {
				result: Result<(), super::system::types::DispatchError>,
			} = 0,
			/// A pure account has been created by new proxy with given
			/// disambiguation index and proxy type.
			PureCreated {
				pure: AccountId,
				who: AccountId,
				proxy_type: super::types::ProxyType,
				disambiguation_index: u16,
			} = 1,
			/// An announcement was placed to make a call in the future.
			Announced {
				real: AccountId,
				proxy: AccountId,
				call_hash: H256,
			} = 2,
			/// A proxy was added.
			ProxyAdded {
				delegator: AccountId,
				delegatee: AccountId,
				proxy_type: super::types::ProxyType,
				delay: u32,
			} = 3,
			/// A proxy was removed.
			ProxyRemoved {
				delegator: AccountId,
				delegatee: AccountId,
				proxy_type: super::types::ProxyType,
				delay: u32,
			} = 4,
		}
		impl Decode for Event {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::ProxyExecuted {
						result: Decode::decode(input)?,
					}),
					1 => Ok(Self::PureCreated {
						pure: Decode::decode(input)?,
						who: Decode::decode(input)?,
						proxy_type: Decode::decode(input)?,
						disambiguation_index: Decode::decode(input)?,
					}),
					2 => Ok(Self::Announced {
						real: Decode::decode(input)?,
						proxy: Decode::decode(input)?,
						call_hash: Decode::decode(input)?,
					}),
					3 => Ok(Self::ProxyAdded {
						delegator: Decode::decode(input)?,
						delegatee: Decode::decode(input)?,
						proxy_type: Decode::decode(input)?,
						delay: Decode::decode(input)?,
					}),
					4 => Ok(Self::ProxyRemoved {
						delegator: Decode::decode(input)?,
						delegatee: Decode::decode(input)?,
						proxy_type: Decode::decode(input)?,
						delay: Decode::decode(input)?,
					}),
					_ => Err("Failed to decode Multisig Event. Unknown variant".into()),
				}
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Call {
			Proxy(Proxy) = Proxy::DISPATCH_INDEX.1,
			AddProxy(AddProxy) = AddProxy::DISPATCH_INDEX.1,
			RemoveProxy(RemoveProxy) = RemoveProxy::DISPATCH_INDEX.1,
			RemoveProxies(RemoveProxies) = RemoveProxies::DISPATCH_INDEX.1,
			CreatePure(CreatePure) = CreatePure::DISPATCH_INDEX.1,
			KillPure(KillPure) = KillPure::DISPATCH_INDEX.1,
		}
		impl Encode for Call {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::Proxy(x) => x.encode_to(dest),
					Self::AddProxy(x) => x.encode_to(dest),
					Self::RemoveProxy(x) => x.encode_to(dest),
					Self::RemoveProxies(x) => x.encode_to(dest),
					Self::CreatePure(x) => x.encode_to(dest),
					Self::KillPure(x) => x.encode_to(dest),
				}
			}
		}
		impl Decode for Call {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					val if val == Proxy::DISPATCH_INDEX.1 => Ok(Self::Proxy(Decode::decode(input)?)),
					val if val == AddProxy::DISPATCH_INDEX.1 => Ok(Self::AddProxy(Decode::decode(input)?)),
					val if val == RemoveProxy::DISPATCH_INDEX.1 => Ok(Self::RemoveProxy(Decode::decode(input)?)),
					val if val == RemoveProxies::DISPATCH_INDEX.1 => Ok(Self::RemoveProxies(Decode::decode(input)?)),
					val if val == CreatePure::DISPATCH_INDEX.1 => Ok(Self::CreatePure(Decode::decode(input)?)),
					val if val == KillPure::DISPATCH_INDEX.1 => Ok(Self::KillPure(Decode::decode(input)?)),
					_ => Err("Failed to decode Proxy Call. Unknown variant".into()),
				}
			}
		}

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
				Ok(Self {
					id,
					force_proxy_type,
					call,
				})
			}
		}
		impl TxDispatchIndex for Proxy {
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
		impl TxDispatchIndex for AddProxy {
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
				Ok(Self {
					delegate,
					proxy_type,
					delay,
				})
			}
		}
		impl TxDispatchIndex for RemoveProxy {
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
		impl TxDispatchIndex for RemoveProxies {
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
				Ok(Self {
					proxy_type,
					delay,
					index,
				})
			}
		}
		impl TxDispatchIndex for CreatePure {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}

		#[derive(Debug, Clone)]
		pub struct KillPure {
			pub spawner: MultiAddress,
			pub proxy_type: super::types::ProxyType,
			pub index: u16,
			pub height: Compact<u32>,
			pub ext_index: Compact<u32>,
		}
		impl Encode for KillPure {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.spawner.encode());
				dest.write(&self.proxy_type.encode());
				dest.write(&self.index.encode());
				dest.write(&self.height.encode());
				dest.write(&self.ext_index.encode());
			}
		}
		impl Decode for KillPure {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let spawner = Decode::decode(input)?;
				let proxy_type = Decode::decode(input)?;
				let index = Decode::decode(input)?;
				let height = Decode::decode(input)?;
				let ext_index = Decode::decode(input)?;
				Ok(Self {
					spawner,
					proxy_type,
					index,
					height,
					ext_index,
				})
			}
		}
		impl TxDispatchIndex for KillPure {
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
				dest.write(&self.height.encode());
				dest.write(&self.index.encode());
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

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Event {
			/// A new multisig operation has begun.
			NewMultisig {
				approving: AccountId,
				multisig: AccountId,
				call_hash: H256,
			} = 0,
			/// A multisig operation has been approved by someone.
			MultisigApproval {
				approving: AccountId,
				timepoint: super::types::Timepoint,
				multisig: AccountId,
				call_hash: H256,
			} = 1,
			/// A multisig operation has been executed.
			MultisigExecuted {
				approving: AccountId,
				timepoint: super::types::Timepoint,
				multisig: AccountId,
				call_hash: H256,
				result: Result<(), super::system::types::DispatchError>,
			} = 2,
			/// A multisig operation has been cancelled.
			MultisigCancelled {
				cancelling: AccountId,
				timepoint: super::types::Timepoint,
				multisig: AccountId,
				call_hash: H256,
			} = 3,
		}
		impl Decode for Event {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::NewMultisig {
						approving: Decode::decode(input)?,
						multisig: Decode::decode(input)?,
						call_hash: Decode::decode(input)?,
					}),
					1 => Ok(Self::MultisigApproval {
						approving: Decode::decode(input)?,
						timepoint: Decode::decode(input)?,
						multisig: Decode::decode(input)?,
						call_hash: Decode::decode(input)?,
					}),
					2 => Ok(Self::MultisigExecuted {
						approving: Decode::decode(input)?,
						timepoint: Decode::decode(input)?,
						multisig: Decode::decode(input)?,
						call_hash: Decode::decode(input)?,
						result: Decode::decode(input)?,
					}),
					3 => Ok(Self::MultisigCancelled {
						cancelling: Decode::decode(input)?,
						timepoint: Decode::decode(input)?,
						multisig: Decode::decode(input)?,
						call_hash: Decode::decode(input)?,
					}),
					_ => Err("Failed to decode Multisig Event. Unknown variant".into()),
				}
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Call {
			AsMultiThreshold1(AsMultiThreshold1) = AsMultiThreshold1::DISPATCH_INDEX.1,
			AsMulti(AsMulti) = AsMulti::DISPATCH_INDEX.1,
			ApproveAsMulti(ApproveAsMulti) = ApproveAsMulti::DISPATCH_INDEX.1,
			CancelAsMulti(CancelAsMulti) = CancelAsMulti::DISPATCH_INDEX.1,
		}
		impl Encode for Call {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::AsMultiThreshold1(x) => x.encode_to(dest),
					Self::AsMulti(x) => x.encode_to(dest),
					Self::ApproveAsMulti(x) => x.encode_to(dest),
					Self::CancelAsMulti(x) => x.encode_to(dest),
				}
			}
		}
		impl Decode for Call {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					val if val == AsMultiThreshold1::DISPATCH_INDEX.1 => {
						Ok(Self::AsMultiThreshold1(Decode::decode(input)?))
					},
					val if val == AsMulti::DISPATCH_INDEX.1 => Ok(Self::AsMulti(Decode::decode(input)?)),
					val if val == ApproveAsMulti::DISPATCH_INDEX.1 => Ok(Self::ApproveAsMulti(Decode::decode(input)?)),
					val if val == CancelAsMulti::DISPATCH_INDEX.1 => Ok(Self::CancelAsMulti(Decode::decode(input)?)),
					_ => Err("Failed to decode Multisig Call. Unknown variant".into()),
				}
			}
		}

		#[derive(Debug, Clone)]
		pub struct AsMultiThreshold1 {
			pub other_signatories: Vec<AccountId>,
			pub call: TransactionCall,
		}
		impl Encode for AsMultiThreshold1 {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.other_signatories.encode());
				dest.write(&self.call.encode());
			}
		}
		impl Decode for AsMultiThreshold1 {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let other_signatories = Decode::decode(input)?;
				let call = Decode::decode(input)?;
				Ok(Self {
					other_signatories,
					call,
				})
			}
		}
		impl TxDispatchIndex for AsMultiThreshold1 {
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
				dest.write(&self.threshold.encode());
				dest.write(&self.other_signatories.encode());
				dest.write(&self.maybe_timepoint.encode());
				dest.write(&self.call.encode());
				dest.write(&self.max_weight.encode());
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
		impl TxDispatchIndex for AsMulti {
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
				dest.write(&self.threshold.encode());
				dest.write(&self.other_signatories.encode());
				dest.write(&self.maybe_timepoint.encode());
				dest.write(&self.call_hash.encode());
				dest.write(&self.max_weight.encode());
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
		impl TxDispatchIndex for ApproveAsMulti {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Debug, Clone)]
		pub struct CancelAsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub maybe_timepoint: super::types::Timepoint,
			pub call_hash: H256,
		}
		impl Encode for CancelAsMulti {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.threshold.encode());
				dest.write(&self.other_signatories.encode());
				dest.write(&self.maybe_timepoint.encode());
				dest.write(&self.call_hash.encode());
			}
		}
		impl Decode for CancelAsMulti {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let threshold = Decode::decode(input)?;
				let other_signatories = Decode::decode(input)?;
				let maybe_timepoint = Decode::decode(input)?;
				let call_hash = Decode::decode(input)?;
				Ok(Self {
					threshold,
					other_signatories,
					maybe_timepoint,
					call_hash,
				})
			}
		}
		impl TxDispatchIndex for CancelAsMulti {
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
		use serde::Deserialize;

		/// Message type used to bridge between Avail & other chains
		#[derive(Debug, Clone, Deserialize)]
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
				dest.write(&self.message.encode());
				dest.write(&self.from.encode());
				dest.write(&self.to.encode());
				dest.write(&self.origin_domain.encode());
				dest.write(&self.destination_domain.encode());
				dest.write(&self.id.encode());
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
				Ok(Self {
					message,
					from,
					to,
					origin_domain,
					destination_domain,
					id,
				})
			}
		}

		/// Possible types of Messages allowed by Avail to bridge to other chains.
		#[derive(Debug, Clone, Deserialize)]
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
				dest.write(&self.slots_per_period.encode());
				dest.write(&self.finality_threshold.encode());
			}
		}
		impl Decode for Configuration {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let slots_per_period = Decode::decode(input)?;
				let finality_threshold = Decode::decode(input)?;
				Ok(Self {
					slots_per_period,
					finality_threshold,
				})
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
				dest.write(&self.function_id.encode());
				dest.write(&self.input.encode());
				dest.write(&self.output.encode());
				dest.write(&self.proof.encode());
				dest.write(&self.slot.encode());
			}
		}
		impl Decode for FulfillCall {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let function_id = Decode::decode(input)?;
				let inputt = Decode::decode(input)?;
				let output = Decode::decode(input)?;
				let proof = Decode::decode(input)?;
				let slot = Decode::decode(input)?;
				Ok(Self {
					function_id,
					input: inputt,
					output,
					proof,
					slot,
				})
			}
		}
		impl TxDispatchIndex for FulfillCall {
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
				dest.write(&self.slot.encode());
				dest.write(&self.addr_message.encode());
				dest.write(&self.account_proof.encode());
				dest.write(&self.storage_proof.encode());
			}
		}
		impl Decode for Execute {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let slot = Decode::decode(input)?;
				let addr_message = Decode::decode(input)?;
				let account_proof = Decode::decode(input)?;
				let storage_proof = Decode::decode(input)?;
				Ok(Self {
					slot,
					addr_message,
					account_proof,
					storage_proof,
				})
			}
		}
		impl TxDispatchIndex for Execute {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Debug, Clone)]
		pub struct SourceChainFroze {
			pub source_chain_id: Compact<u32>,
			pub frozen: bool,
		}
		impl Encode for SourceChainFroze {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.source_chain_id.encode());
				dest.write(&self.frozen.encode());
			}
		}
		impl Decode for SourceChainFroze {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let source_chain_id = Decode::decode(input)?;
				let frozen = Decode::decode(input)?;
				Ok(Self {
					source_chain_id,
					frozen,
				})
			}
		}
		impl TxDispatchIndex for SourceChainFroze {
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
				dest.write(&self.slot.encode());
				dest.write(&self.message.encode());
				dest.write(&self.to.encode());
				dest.write(&self.domain.encode());
			}
		}
		impl Decode for SendMessage {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let slot = Decode::decode(input)?;
				let message = Decode::decode(input)?;
				let to = Decode::decode(input)?;
				let domain = Decode::decode(input)?;
				Ok(Self {
					slot,
					message,
					to,
					domain,
				})
			}
		}
		impl TxDispatchIndex for SendMessage {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Debug, Clone)]
		pub struct SetPoseidonHash {
			pub period: Compact<u64>,
			pub poseidon_hash: Vec<u8>,
		}
		impl Encode for SetPoseidonHash {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.period.encode());
				dest.write(&self.poseidon_hash.encode());
			}
		}
		impl Decode for SetPoseidonHash {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let period = Decode::decode(input)?;
				let poseidon_hash = Decode::decode(input)?;
				Ok(Self { period, poseidon_hash })
			}
		}
		impl TxDispatchIndex for SetPoseidonHash {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}

		#[derive(Debug, Clone)]
		pub struct SetBroadcaster {
			pub broadcaster_domain: Compact<u32>,
			pub broadcaster: H256,
		}
		impl Encode for SetBroadcaster {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.broadcaster_domain.encode());
				dest.write(&self.broadcaster.encode());
			}
		}
		impl Decode for SetBroadcaster {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let broadcaster_domain = Decode::decode(input)?;
				let broadcaster = Decode::decode(input)?;
				Ok(Self {
					broadcaster_domain,
					broadcaster,
				})
			}
		}
		impl TxDispatchIndex for SetBroadcaster {
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
		impl TxDispatchIndex for SetWhitelistedDomains {
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
		impl TxDispatchIndex for SetConfiguration {
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
		impl TxDispatchIndex for SetFunctionIds {
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
		impl TxDispatchIndex for SetStepVerificationKey {
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
		impl TxDispatchIndex for SetRotateVerificationKey {
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
		impl TxDispatchIndex for SetUpdater {
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
		impl TxDispatchIndex for Fulfill {
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
		impl TxDispatchIndex for SetSp1VerificationKey {
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
		impl TxDispatchIndex for SetSyncCommitteeHash {
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
		impl TxDispatchIndex for EnableMock {
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
		impl TxDispatchIndex for MockFulfill {
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
		#[derive(Debug, Clone, DecodeAsType, EncodeAsType)]
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
				Ok(Self {
					nonce,
					consumers,
					providers,
					sufficients,
					data,
				})
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
				Ok(Self {
					weight,
					class,
					pays_fee,
					fee_modifier,
				})
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

		pub fn account_iter() -> StaticAddress<(), super::system::types::AccountInfo, (), Yes, Yes> {
			let address = StaticAddress::new_static("System", "Account", (), Default::default());
			address.unvalidated()
		}

		pub fn account(
			account_id: &AccountId,
		) -> StaticAddress<StaticStorageKey<AccountId>, super::system::types::AccountInfo, Yes, Yes, ()> {
			let address = StaticAddress::new_static(
				"System",
				"Account",
				StaticStorageKey::new(account_id),
				Default::default(),
			);
			address.unvalidated()
		}
	}

	pub mod events {
		use super::*;

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Event {
			ExtrinsicSuccess {
				dispatch_info: super::types::DispatchInfo,
			} = 0,
			ExtrinsicFailed {
				dispatch_error: super::types::DispatchError,
				dispatch_info: super::types::DispatchInfo,
			} = 1,
		}
		/* 		impl Encode for Event {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::ExtrinsicSuccess { dispatch_info } => dispatch_info.encode_to(dest),
					Self::ExtrinsicFailed {
						dispatch_error,
						dispatch_info,
					} => {
						dispatch_error.encode_to(dest);
						dispatch_info.encode_to(dest);
					},
				}
			}
		} */
		impl Decode for Event {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					0 => Ok(Self::ExtrinsicSuccess {
						dispatch_info: Decode::decode(input)?,
					}),
					1 => Ok(Self::ExtrinsicFailed {
						dispatch_error: Decode::decode(input)?,
						dispatch_info: Decode::decode(input)?,
					}),
					_ => Err("Failed to decode System Event. Unknown variant".into()),
				}
			}
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Debug, Clone)]
		#[repr(u8)]
		pub enum Call {
			Remark(Remark) = Remark::DISPATCH_INDEX.1,
			SetCode(SetCode) = SetCode::DISPATCH_INDEX.1,
			SetCodeWithoutChecks(SetCodeWithoutChecks) = SetCodeWithoutChecks::DISPATCH_INDEX.1,
			RemarkWithEvent(RemarkWithEvent) = RemarkWithEvent::DISPATCH_INDEX.1,
		}
		impl Encode for Call {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				variant.encode_to(dest);
				match self {
					Self::Remark(x) => x.encode_to(dest),
					Self::SetCode(x) => x.encode_to(dest),
					Self::SetCodeWithoutChecks(x) => x.encode_to(dest),
					Self::RemarkWithEvent(x) => x.encode_to(dest),
				}
			}
		}
		impl Decode for Call {
			fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
				let variant = u8::decode(input)?;
				match variant {
					val if val == Remark::DISPATCH_INDEX.1 => Ok(Self::Remark(Decode::decode(input)?)),
					val if val == SetCode::DISPATCH_INDEX.1 => Ok(Self::SetCode(Decode::decode(input)?)),
					val if val == SetCodeWithoutChecks::DISPATCH_INDEX.1 => {
						Ok(Self::SetCodeWithoutChecks(Decode::decode(input)?))
					},
					val if val == RemarkWithEvent::DISPATCH_INDEX.1 => {
						Ok(Self::RemarkWithEvent(Decode::decode(input)?))
					},
					_ => Err("Failed to decode System Call. Unknown variant".into()),
				}
			}
		}

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
		impl TxDispatchIndex for Remark {
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
		impl TxDispatchIndex for SetCode {
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
		impl TxDispatchIndex for SetCodeWithoutChecks {
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
		impl TxDispatchIndex for RemarkWithEvent {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 7);
		}
	}
}
