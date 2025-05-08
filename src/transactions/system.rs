use crate::client::Client;
use crate::config::{AccountId, AccountInfo};
use primitive_types::H256;
use subxt_core::storage::address::Address;
use subxt_core::utils::Yes;

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

pub async fn full<'address, Addr>(
	metadata: &subxt_core::Metadata,
	address: &'address Addr,
	client: &Client,
	block_hash: H256,
) -> Addr::Target
where
	Addr: Address<IsFetchable = Yes, IsDefaultable = Yes> + 'address,
{
	let key = subxt_core::storage::get_address_bytes(address, &metadata).unwrap();
	let data = client.rpc_state_get_storage(key, Some(block_hash)).await.unwrap();
	subxt_core::storage::decode_value(&mut &*data, address, &metadata).unwrap()
}
