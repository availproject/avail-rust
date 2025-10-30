use avail_rust_client::prelude::*;
use avail_rust_core::avail::staking::types::ValidatorPrefs;

// Custom Double Map
pub struct StakingErasValidatorPrefs;
impl StorageDoubleMap for StakingErasValidatorPrefs {
	type KEY1 = u32;
	type KEY2 = AccountId;
	type VALUE = ValidatorPrefs;

	const KEY1_HASHER: StorageHasher = StorageHasher::Twox64Concat;
	const KEY2_HASHER: StorageHasher = StorageHasher::Twox64Concat;
	const PALLET_NAME: &str = "Staking";
	const STORAGE_NAME: &str = "ErasValidatorPrefs";
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;
	let rpc_client = &client.rpc_client;

	// Fetching Staking::ErasValidatorPrefs - Storage Double Map
	let block_hash =
		H256::from_str("0xd81274fdfdcca5cd764d301f4d34aafb797ff466bd73bd2fc4a3ca5108ac2f6a").expect("Should be ok");

	let era_index = 582;
	let account_id = AccountId::from_str("5HpkbR8i5cf87grRKxmYssuVzuVeXaakv4TJLyMEQDdvfxJa").expect("Should be ok");
	let pref = StakingErasValidatorPrefs::fetch(rpc_client, &era_index, &account_id, Some(block_hash))
		.await?
		.expect("Should be ok");
	println!(
		"Era Index: {}, Account Id: {}, Blocked: {}, Commission: {}",
		era_index, account_id, pref.blocked, pref.commission
	);
	println!("");

	// Iterating over Staking::ErasValidatorPrefs
	let mut iter = StakingErasValidatorPrefs::iter(rpc_client.clone(), &era_index, block_hash);
	for _ in 0..2 {
		// You can fetch just the value...
		let (account_id, pref) = iter.next().await?.expect("Should be there");
		println!("Account Id: {}, Blocked: {}, Commission: {}", account_id, pref.blocked, pref.commission);

		// ...or both the value and the key
		let (era_index, account_id, pref) = iter.next_key_value().await?.expect("Should be there");
		println!(
			"Era Index: {}, Account Id: {}, Blocked: {}, Commission: {}",
			era_index, account_id, pref.blocked, pref.commission
		);
	}

	Ok(())
}

/*
	Expected Output:

	Era Index: 582, Account Id: 5HpkbR8i5cf87grRKxmYssuVzuVeXaakv4TJLyMEQDdvfxJa, Blocked: false, Commission: 100000000

	Account Id: 5DeVoYGJzAKuuy3tKpQPPK15fUCdidNziJ2EPNxizPQKJJKy, Blocked: false, Commission: 10000000
	Era Index: 582, Account Id: 5DeVoYGKbtY2mKDCB9rXVnw6inDcmxTQzvYvSoUm88Z7wWX6, Blocked: false Commission: 50000000
	Account Id: 5DeVoYGKvcTtpMzU22WAG85SPjG3pNvFfa8KxmUGt2L5aJop, Blocked: false, Commission: 100000000
	Era Index: 582, Account Id: 5DeVoYGLTsFiSd7WfUjq6pvyRh852jWL8Wn7WW76K4Un85J3, Blocked: false, Commission: 100000000
*/
