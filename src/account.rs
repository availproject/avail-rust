use crate::{
	avail, avail::system::storage::types::account::Account, error::ClientError, rpc, AOnlineClient,
	AccountId,
};
use primitive_types::H256;
use subxt::backend::rpc::RpcClient;

pub async fn fetch_nonce_state(
	online_client: &AOnlineClient,
	rpc_client: &RpcClient,
	address: &str,
	block_hash: Option<H256>,
) -> Result<u32, ClientError> {
	let account = account_id_from_str(address)?;
	let block_hash = match block_hash {
		Some(x) => x,
		None => rpc::chain::get_block_hash(rpc_client, None).await?,
	};
	let block = online_client.blocks().at(block_hash).await?;

	Ok(block.account_nonce(&account).await? as u32)
}

pub async fn fetch_nonce_node(client: &RpcClient, address: &str) -> Result<u32, ClientError> {
	let account = account_id_from_str(address)?;
	rpc::system::account_next_index(client, account.to_string()).await
}

pub async fn fetch_app_keys(
	online_client: &AOnlineClient,
	rpc_client: &RpcClient,
	account_id: AccountId,
) -> Result<Vec<(String, u32)>, String> {
	let block_hash = rpc::chain::get_block_hash(rpc_client, None).await;
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

pub async fn fetch_app_ids(
	online_client: &AOnlineClient,
	rpc_client: &RpcClient,
	account_id: AccountId,
) -> Result<Vec<u32>, String> {
	let keys = match fetch_app_keys(online_client, rpc_client, account_id).await {
		Ok(k) => k,
		Err(e) => return Err(e),
	};

	Ok(keys.into_iter().map(|v| v.1).collect())
}

pub async fn fetch_balance(
	online_client: &AOnlineClient,
	rpc_client: &RpcClient,
	account_id: AccountId,
) -> Result<Account, String> {
	let block_hash = rpc::chain::get_block_hash(rpc_client, None).await;
	let block_hash = block_hash.map_err(|e| e.to_string())?;

	let storage = online_client.storage().at(block_hash);
	let address = avail::storage().system().account(account_id);
	let result = storage
		.fetch_or_default(&address)
		.await
		.map_err(|e| e.to_string())?;

	Ok(result)
}

pub fn account_id_from_str(value: &str) -> Result<AccountId, String> {
	value.parse().map_err(|e| std::format!("{:?}", e))
}
