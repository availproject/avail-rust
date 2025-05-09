pub mod avail {
	pub use subxt_core::storage::address::{StaticAddress, StaticStorageKey};
	pub use subxt_core::utils::Yes;

	use crate::config::{AccountId, AccountInfo};

	pub fn storage() -> Storage {
		Storage
	}

	pub struct Storage;
	impl Storage {
		pub fn system(&self) -> StorageSystem {
			StorageSystem
		}
	}

	pub struct StorageSystem;
	impl StorageSystem {
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
