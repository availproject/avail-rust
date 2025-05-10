use crate::config::{AccountId, AccountInfo};
use crate::primitives;
use codec::{Decode, Encode};
use subxt_core::storage::address::{StaticAddress, StaticStorageKey};
use subxt_core::utils::Yes;

pub trait TxDispatchIndex {
	// Pallet ID, Call ID
	const DISPATCH_INDEX: (u8, u8);
}

pub mod avail {
	pub fn tx() -> TransactionsApi {
		TransactionsApi
	}

	pub fn storage() -> StorageApi {
		StorageApi
	}

	pub struct TransactionsApi;
	impl TransactionsApi {
		pub fn data_availability(&self) -> super::transactions::DataAvailability {
			super::transactions::DataAvailability
		}
	}

	pub struct StorageApi;
	impl StorageApi {
		pub fn system(&self) -> super::storage::System {
			super::storage::System
		}
	}
}

pub mod types {
	use super::*;
	pub mod data_availability {
		use super::*;

		pub struct CreateApplicationKey;
		impl TxDispatchIndex for CreateApplicationKey {
			const DISPATCH_INDEX: (u8, u8) = (29, 0);
		}
		impl CreateApplicationKey {
			pub fn to_call(key: Vec<u8>) -> primitives::TransactionCall {
				primitives::TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, key.encode())
			}
		}

		pub struct SubmitData;
		impl TxDispatchIndex for SubmitData {
			const DISPATCH_INDEX: (u8, u8) = (29, 1);
		}
		impl SubmitData {
			pub fn to_call(data: Vec<u8>) -> primitives::TransactionCall {
				primitives::TransactionCall::new(Self::DISPATCH_INDEX.0, Self::DISPATCH_INDEX.1, data.encode())
			}
		}
	}
}

pub mod transactions {
	use super::*;

	pub struct DataAvailability;
	impl DataAvailability {
		pub fn create_application_key(&self, key: Vec<u8>) -> primitives::TransactionCall {
			types::data_availability::CreateApplicationKey::to_call(key)
		}

		pub fn submit_data(&self, data: Vec<u8>) -> primitives::TransactionCall {
			types::data_availability::SubmitData::to_call(data)
		}
	}
}

pub mod storage {
	use super::*;

	pub struct System;
	impl System {
		pub fn account(
			&self,
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
