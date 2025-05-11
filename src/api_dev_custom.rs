use crate::config::MultiAddress;
use crate::config::{AccountId, AccountInfo};
use crate::primitives::TransactionCall;
use codec::Encode;
use primitive_types::H256;
use subxt_core::storage::address::{StaticAddress, StaticStorageKey};
use subxt_core::utils::Yes;

pub trait TxDispatchIndex {
	// Pallet ID, Call ID
	const DISPATCH_INDEX: (u8, u8);
}

pub trait TransactionCallLike {
	fn to_call(&self) -> TransactionCall;
}

impl<T: TxDispatchIndex + Encode> TransactionCallLike for T {
	fn to_call(&self) -> TransactionCall {
		TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, self.encode())
	}
}

pub mod data_availability {
	use super::*;
	const PALLET_ID: u8 = 29;

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct CreateApplicationKey {
			pub key: Vec<u8>,
		}
		impl TxDispatchIndex for CreateApplicationKey {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct SubmitData {
			pub data: Vec<u8>,
		}
		impl TxDispatchIndex for SubmitData {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
	}
}

pub mod balances {
	use super::*;
	const PALLET_ID: u8 = 6;

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct TransferAllowDeath {
			pub dest: MultiAddress,
			#[codec(compact)]
			pub amount: u128,
		}
		impl TxDispatchIndex for TransferAllowDeath {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct TransferKeepAlive {
			pub dest: MultiAddress,
			#[codec(compact)]
			pub amount: u128,
		}
		impl TxDispatchIndex for TransferKeepAlive {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Encode)]
		pub struct TransferAll {
			pub dest: MultiAddress,
			pub keep_alive: bool,
		}
		impl TxDispatchIndex for TransferAll {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
	}
}

pub mod utility {
	use super::*;
	const PALLET_ID: u8 = 1;

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct Batch {
			pub calls: Vec<TransactionCall>,
		}
		impl TxDispatchIndex for Batch {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct BatchAll {
			pub calls: Vec<TransactionCall>,
		}
		impl TxDispatchIndex for BatchAll {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Encode)]
		pub struct ForceBatch {
			pub calls: Vec<TransactionCall>,
		}
		impl TxDispatchIndex for ForceBatch {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
	}
}

pub mod proxy {
	use super::*;
	const PALLET_ID: u8 = 40;

	pub mod types {
		use super::*;

		#[derive(Debug, Encode, Clone, Copy)]
		#[repr(u8)]
		pub enum ProxyType {
			Any = 0,
			NonTransfer = 1,
			Governance = 2,
			Staking = 3,
			IdentityJudgement = 4,
			NominationPools = 5,
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct Proxy {
			pub id: MultiAddress,
			pub force_proxy_type: Option<super::types::ProxyType>,
			pub call: TransactionCall,
		}
		impl TxDispatchIndex for Proxy {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct AddProxy {
			pub id: MultiAddress,
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
		}
		impl TxDispatchIndex for AddProxy {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Encode)]
		pub struct RemoveProxy {
			pub delegate: MultiAddress,
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
		}
		impl TxDispatchIndex for RemoveProxy {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Encode)]
		pub struct RemoveProxies;
		impl TxDispatchIndex for RemoveProxies {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}

		#[derive(Encode)]
		pub struct CreatePure {
			pub proxy_type: super::types::ProxyType,
			pub delay: u32,
			pub index: u16,
		}
		impl TxDispatchIndex for CreatePure {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}

		#[derive(Encode)]
		pub struct KillPure {
			pub spawner: MultiAddress,
			pub proxy_type: super::types::ProxyType,
			pub index: u16,
			#[codec(compact)]
			pub height: u32,
			#[codec(compact)]
			pub ext_index: u32,
		}
		impl TxDispatchIndex for KillPure {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 5);
		}
	}
}

pub mod multisig {
	use super::*;
	const PALLET_ID: u8 = 34;

	pub mod types {
		use super::*;
		pub use crate::from_substrate::Weight;

		#[derive(Debug, Encode, Clone, Copy)]
		pub struct Timepoint {
			height: u32,
			index: u32,
		}
	}

	pub mod tx {
		use super::*;

		#[derive(Encode)]
		pub struct AsMultiThreshold1 {
			pub other_signatories: Vec<AccountId>,
			pub call: TransactionCall,
		}
		impl TxDispatchIndex for AsMultiThreshold1 {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}

		#[derive(Encode)]
		pub struct AsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub maybe_timepoint: Option<super::types::Timepoint>,
			pub call: TransactionCall,
			pub max_weight: super::types::Weight,
		}
		impl TxDispatchIndex for AsMulti {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}

		#[derive(Encode)]
		pub struct ApproveAsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub maybe_timepoint: Option<super::types::Timepoint>,
			pub call_hash: H256,
			pub max_weight: super::types::Weight,
		}
		impl TxDispatchIndex for ApproveAsMulti {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 2);
		}

		#[derive(Encode)]
		pub struct CancelAsMulti {
			pub threshold: u16,
			pub other_signatories: Vec<AccountId>,
			pub maybe_timepoint: super::types::Timepoint,
			pub call_hash: H256,
		}
		impl TxDispatchIndex for CancelAsMulti {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}
	}
}

pub mod system {
	use super::*;
	pub mod storage {
		use super::*;
		pub fn account(
			account_id: &AccountId,
		) -> StaticAddress<StaticStorageKey<AccountId>, AccountInfo, Yes, Yes, ()> {
			let address = StaticAddress::new_static(
				"System",
				"Account",
				StaticStorageKey::new(account_id),
				Default::default(),
			);
			address.unvalidated()
		}
	}
}
