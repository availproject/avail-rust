use crate::{
	avail::{self},
	AccountId, Client,
};
use primitive_types::H256;

pub async fn app_keys(client: &Client, account_id: AccountId, block_hash: H256) -> Result<Vec<(String, u32)>, String> {
	let storage = client.subxt_storage().at(block_hash);
	let address = avail::storage().data_availability().app_keys_iter();

	let mut app_keys = storage.iter(address).await.map_err(|e| e.to_string())?;

	let mut result = Vec::new();
	while let Some(Ok(kv)) = app_keys.next().await {
		let key = (kv.key_bytes[49..]).to_vec();
		let key = String::from_utf8(key).unwrap();

		if kv.value.owner == account_id {
			result.push((key.clone(), kv.value.id.0));
		}
	}

	result.sort_by(|a, b| a.1.cmp(&b.1));

	Ok(result)
}

pub async fn app_ids(client: &Client, account_id: AccountId, block_hash: H256) -> Result<Vec<u32>, String> {
	let keys = match app_keys(client, account_id, block_hash).await {
		Ok(k) => k,
		Err(e) => return Err(e),
	};

	Ok(keys.into_iter().map(|v| v.1).collect())
}
