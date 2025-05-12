use crate::primitives::{AccountId, MultiAddress, TransactionCall};
use codec::Compact;
use codec::{Decode, Encode};
use primitive_types::H256;
use scale_decode::DecodeAsType;
use scale_encode::EncodeAsType;
use subxt_core::storage::address::{StaticAddress, StaticStorageKey};
use subxt_core::utils::Yes;

pub trait TxDispatchIndex {
	// Pallet ID, Call ID
	const DISPATCH_INDEX: (u8, u8);
}

pub trait TransactionCallLike {
	fn to_call(&self) -> TransactionCall;
}

impl<T: TxDispatchIndex + Encode + Decode> TransactionCallLike for T {
	fn to_call(&self) -> TransactionCall {
		TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, self.encode())
	}
}

#[derive(Clone)]
#[repr(u8)]
pub enum RuntimeCall {
	DataAvailability(data_availability::tx::Call) = data_availability::PALLET_ID,
	Balances(balances::tx::Call) = balances::PALLET_ID,
	Utility(utility::tx::Call) = utility::PALLET_ID,
	Proxy(proxy::tx::Call) = proxy::PALLET_ID,
	Multisig(multisig::tx::Call) = multisig::PALLET_ID,
	System(system::tx::Call) = system::PALLET_ID,
}
impl Encode for RuntimeCall {
	fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
		let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
		dest.write(&variant.encode());
		match self {
			Self::DataAvailability(x) => dest.write(&x.encode()),
			Self::Balances(x) => dest.write(&x.encode()),
			Self::Utility(x) => dest.write(&x.encode()),
			Self::Proxy(x) => dest.write(&x.encode()),
			Self::Multisig(x) => dest.write(&x.encode()),
			Self::System(x) => dest.write(&x.encode()),
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

pub mod data_availability {
	use super::*;
	pub const PALLET_ID: u8 = 29;

	pub mod tx {
		use super::*;

		#[derive(Clone)]
		#[repr(u8)]
		pub enum Call {
			CreateApplicationKey(CreateApplicationKey) = CreateApplicationKey::DISPATCH_INDEX.1,
			SubmitData(SubmitData) = SubmitData::DISPATCH_INDEX.1,
		}
		impl Encode for Call {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				dest.write(&variant.encode());
				match self {
					Self::CreateApplicationKey(x) => dest.write(&x.encode()),
					Self::SubmitData(x) => dest.write(&x.encode()),
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

		#[derive(Clone)]
		pub struct CreateApplicationKey {
			pub key: Vec<u8>,
		}
		impl Encode for CreateApplicationKey {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.key.encode());
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

		#[derive(Clone)]
		pub struct SubmitData {
			pub data: Vec<u8>,
		}
		impl Encode for SubmitData {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.data.encode());
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
				dest.write(&self.free.encode());
				dest.write(&self.reserved.encode());
				dest.write(&self.frozen.encode());
				dest.write(&self.flags.encode());
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
	}

	pub mod tx {
		use super::*;

		#[derive(Clone)]
		#[repr(u8)]
		pub enum Call {
			TransferAllowDeath(TransferAllowDeath) = TransferAllowDeath::DISPATCH_INDEX.1,
			TransferKeepAlive(TransferKeepAlive) = TransferKeepAlive::DISPATCH_INDEX.1,
			TransferAll(TransferAll) = TransferAll::DISPATCH_INDEX.1,
		}
		impl Encode for Call {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				dest.write(&variant.encode());
				match self {
					Self::TransferAllowDeath(x) => dest.write(&x.encode()),
					Self::TransferKeepAlive(x) => dest.write(&x.encode()),
					Self::TransferAll(x) => dest.write(&x.encode()),
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

	pub mod tx {
		use super::*;

		#[derive(Clone)]
		#[repr(u8)]
		pub enum Call {
			Batch(Batch) = Batch::DISPATCH_INDEX.1,
			BatchAll(BatchAll) = BatchAll::DISPATCH_INDEX.1,
			ForceBatch(ForceBatch) = ForceBatch::DISPATCH_INDEX.1,
		}
		impl Encode for Call {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				let variant: u8 = unsafe { *<*const _>::from(self).cast::<u8>() };
				dest.write(&variant.encode());
				match self {
					Self::Batch(x) => dest.write(&x.encode()),
					Self::BatchAll(x) => dest.write(&x.encode()),
					Self::ForceBatch(x) => dest.write(&x.encode()),
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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
				dest.write(&variant.encode());
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

	pub mod tx {
		use super::*;

		#[derive(Clone)]
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
				dest.write(&variant.encode());
				match self {
					Self::Proxy(x) => dest.write(&x.encode()),
					Self::AddProxy(x) => dest.write(&x.encode()),
					Self::RemoveProxy(x) => dest.write(&x.encode()),
					Self::RemoveProxies(x) => dest.write(&x.encode()),
					Self::CreatePure(x) => dest.write(&x.encode()),
					Self::KillPure(x) => dest.write(&x.encode()),
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

	pub mod tx {
		use super::*;

		#[derive(Clone)]
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
				dest.write(&variant.encode());
				match self {
					Self::AsMultiThreshold1(x) => dest.write(&x.encode()),
					Self::AsMulti(x) => dest.write(&x.encode()),
					Self::ApproveAsMulti(x) => dest.write(&x.encode()),
					Self::CancelAsMulti(x) => dest.write(&x.encode()),
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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
				dest.write(&variant.encode());
				match self {
					Message::ArbitraryMessage(items) => {
						dest.write(&items.encode());
					},
					Message::FungibleToken { asset_id, amount } => {
						dest.write(&asset_id.encode());
						dest.write(&amount.encode());
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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

		#[derive(Clone)]
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
				dest.write(&self.nonce.encode());
				dest.write(&self.consumers.encode());
				dest.write(&self.providers.encode());
				dest.write(&self.sufficients.encode());
				dest.write(&self.data.encode());
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
	}

	pub mod storage {
		use super::*;
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

	pub mod tx {
		use super::*;

		#[derive(Clone)]
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
				dest.write(&variant.encode());
				match self {
					Self::Remark(x) => dest.write(&x.encode()),
					Self::SetCode(x) => dest.write(&x.encode()),
					Self::SetCodeWithoutChecks(x) => dest.write(&x.encode()),
					Self::RemarkWithEvent(x) => dest.write(&x.encode()),
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

		#[derive(Clone)]
		pub struct Remark {
			pub remark: Vec<u8>,
		}
		impl Encode for Remark {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.remark.encode());
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

		#[derive(Clone)]
		pub struct SetCode {
			pub code: Vec<u8>,
		}
		impl Encode for SetCode {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.code.encode());
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

		#[derive(Clone)]
		pub struct SetCodeWithoutChecks {
			pub code: Vec<u8>,
		}
		impl Encode for SetCodeWithoutChecks {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.code.encode());
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

		#[derive(Clone)]
		pub struct RemarkWithEvent {
			pub remark: Vec<u8>,
		}
		impl Encode for RemarkWithEvent {
			fn encode_to<T: codec::Output + ?Sized>(&self, dest: &mut T) {
				dest.write(&self.remark.encode());
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

pub mod utils {
	use std::array::TryFromSliceError;

	#[derive(Debug, Clone)]
	pub struct SessionKeys {
		pub babe: [u8; 32],
		pub grandpa: [u8; 32],
		pub im_online: [u8; 32],
		pub authority_discovery: [u8; 32],
	}

	impl TryFrom<&[u8]> for SessionKeys {
		type Error = String;

		fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
			if value.len() != 128 {
				return Err(String::from(
					"Session keys len cannot have length be more or less than 128",
				));
			}

			let err = |e: TryFromSliceError| e.to_string();

			let babe: [u8; 32] = value[0..32].try_into().map_err(err)?;
			let grandpa: [u8; 32] = value[32..64].try_into().map_err(err)?;
			let im_online: [u8; 32] = value[64..96].try_into().map_err(err)?;
			let authority_discovery: [u8; 32] = value[96..128].try_into().map_err(err)?;
			Ok(Self {
				babe,
				grandpa,
				im_online,
				authority_discovery,
			})
		}
	}
}
