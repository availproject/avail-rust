use crate::{avail, error::ClientError, rpcs::get_block_hash, utils, AOnlineClient, AccountId};
use subxt::backend::rpc::reconnecting_rpc_client::RpcClient;

pub async fn fetch_account_nonce_state(
	online_client: &AOnlineClient,
	rpc_client: &RpcClient,
	address: &str,
) -> Result<u32, ClientError> {
	utils::fetch_nonce_state(online_client, rpc_client, address).await
}

pub async fn fetch_account_nonce_node(
	rpc_client: &RpcClient,
	address: &str,
) -> Result<u32, ClientError> {
	utils::fetch_nonce_node(rpc_client, address).await
}

pub async fn fetch_account_app_keys(
	online_client: &AOnlineClient,
	rpc_client: &RpcClient,
	account_id: AccountId,
) -> Result<Vec<(String, u32)>, String> {
	let block_hash = get_block_hash(rpc_client, None).await;
	let block_hash = block_hash.map_err(|e| e.to_string())?;

	let storage = online_client.storage().at(block_hash);
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

pub async fn fetch_account_app_ids(
	online_client: &AOnlineClient,
	rpc_client: &RpcClient,
	account_id: AccountId,
) -> Result<Vec<u32>, String> {
	let keys = match fetch_account_app_keys(online_client, rpc_client, account_id).await {
		Ok(k) => k,
		Err(e) => return Err(e),
	};

	Ok(keys.into_iter().map(|v| v.1).collect())
}
