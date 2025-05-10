pub mod avail {
	use codec::{Decode, Encode};
	use subxt_core::blocks::StaticExtrinsic;
	use subxt_core::ext::{scale_decode::DecodeAsType, scale_encode::EncodeAsType};
	use subxt_core::storage::address::{StaticAddress, StaticStorageKey};
	use subxt_core::tx::payload::DefaultPayload;
	use subxt_core::utils::Yes;

	use crate::config::{AccountId, AccountInfo};

	pub fn tx() -> transaction::Api {
		transaction::Api
	}

	pub fn storage() -> storages::Api {
		storages::Api
	}

	pub mod data_availability {
		pub use super::*;

		pub mod calls {
			pub use super::*;
			use types::*;

			pub mod types {
				pub use super::*;

				#[derive(Decode, Encode, DecodeAsType, EncodeAsType, Clone, Debug, Eq, PartialEq)]
				#[codec (crate = codec)]
				#[decode_as_type(crate_path = ":: subxt_core :: ext :: scale_decode")]
				#[encode_as_type(crate_path = ":: subxt_core :: ext :: scale_encode")]
				pub struct CreateApplicationKey {
					pub key: Vec<u8>,
				}
				impl StaticExtrinsic for CreateApplicationKey {
					const PALLET: &'static str = "DataAvailability";
					const CALL: &'static str = "create_application_key";
				}
				pub fn payload_crate_application_key(key: Vec<u8>) -> DefaultPayload<CreateApplicationKey> {
					DefaultPayload::<CreateApplicationKey>::new(
						CreateApplicationKey::PALLET,
						CreateApplicationKey::CALL,
						CreateApplicationKey { key },
					)
				}

				#[derive(Clone, Decode, Encode)]
				pub struct SubmitData {
					pub data: Vec<u8>,
				}
			}

			pub struct DataAvailability;
			impl DataAvailability {
				pub fn create_application_key(&self, key: Vec<u8>) -> crate::primitives::transaction::TransactionCall {
					crate::primitives::transaction::TransactionCall {
						pallet_id: 29,
						call_id: 0,
						data: key.encode(),
					}
				}
				pub fn submit_data(&self, data: Vec<u8>) -> crate::primitives::transaction::TransactionCall {
					crate::primitives::transaction::TransactionCall {
						pallet_id: 29,
						call_id: 1,
						data: data.encode(),
					}
				}
			}
		}
	}

	pub mod transaction {
		pub use super::data_availability::calls::DataAvailability;
		pub use super::*;

		pub struct Api;
		impl Api {
			pub fn data_availability(&self) -> DataAvailability {
				DataAvailability
			}
		}
	}

	pub mod storages {
		pub use super::*;

		pub struct Api;
		impl Api {
			pub fn system(&self) -> System {
				System
			}
		}

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
}
