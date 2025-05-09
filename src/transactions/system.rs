#[cfg(not(feature = "subxt"))]
use subxt_core::storage::address::Address;

#[cfg(not(feature = "subxt_metadata"))]
use crate::config::{AccountId, AccountInfo};

#[cfg(not(feature = "subxt_metadata"))]
pub fn account(
	account_id: &AccountId,
) -> subxt_core::storage::address::StaticAddress<
	subxt_core::storage::address::StaticStorageKey<AccountId>,
	AccountInfo,
	subxt_core::utils::Yes,
	subxt_core::utils::Yes,
	(),
> {
	let address = subxt_core::storage::address::StaticAddress::new_static(
		"System",
		"Account",
		subxt_core::storage::address::StaticStorageKey::new(account_id),
		Default::default(),
	);
	address.unvalidated()
}

#[cfg(not(feature = "subxt"))]
pub async fn full<'address, Addr>(
	metadata: &subxt_core::Metadata,
	address: &'address Addr,
	client: &crate::client::Client,
	block_hash: primitive_types::H256,
) -> Addr::Target
where
	Addr: Address<IsFetchable = subxt_core::utils::Yes, IsDefaultable = subxt_core::utils::Yes> + 'address,
{
	let key = subxt_core::storage::get_address_bytes(address, &metadata).unwrap();
	let data = client.rpc_state_get_storage(key, Some(block_hash)).await.unwrap();
	subxt_core::storage::decode_value(&mut &*data, address, &metadata).unwrap()
}
