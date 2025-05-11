use crate::config::{AccountId, AccountInfo};
use crate::primitives;
use codec::Encode;
use subxt_core::storage::address::{StaticAddress, StaticStorageKey};
use subxt_core::utils::Yes;

pub trait TxDispatchIndex {
	// Pallet ID, Call ID
	const DISPATCH_INDEX: (u8, u8);
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
		impl CreateApplicationKey {
			pub fn to_call(&self) -> primitives::TransactionCall {
				primitives::TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, self.encode())
			}
		}

		#[derive(Encode)]
		pub struct SubmitData {
			pub data: Vec<u8>,
		}
		impl TxDispatchIndex for SubmitData {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 1);
		}
		impl SubmitData {
			pub fn to_call(&self) -> primitives::TransactionCall {
				primitives::TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, self.encode())
			}
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
			pub dest: AccountId,
			#[codec(compact)]
			pub amount: u128,
		}
		impl TxDispatchIndex for TransferAllowDeath {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 0);
		}
		impl TransferAllowDeath {
			pub fn to_call(&self) -> primitives::TransactionCall {
				primitives::TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, self.encode())
			}
		}

		#[derive(Encode)]
		pub struct TransferKeepAlive {
			pub dest: AccountId,
			#[codec(compact)]
			pub amount: u128,
		}
		impl TxDispatchIndex for TransferKeepAlive {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 3);
		}
		impl TransferKeepAlive {
			pub fn to_call(&self) -> primitives::TransactionCall {
				primitives::TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, self.encode())
			}
		}

		#[derive(Encode)]
		pub struct TransferAll {
			pub dest: AccountId,
			pub keep_alive: bool,
		}
		impl TxDispatchIndex for TransferAll {
			const DISPATCH_INDEX: (u8, u8) = (PALLET_ID, 4);
		}
		impl TransferAll {
			pub fn to_call(&self) -> primitives::TransactionCall {
				primitives::TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, self.encode())
			}
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
