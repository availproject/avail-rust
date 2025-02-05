use crate::{
	avail::{self, system::storage::types::account::Account},
	error::ClientError,
	rpc, AccountId, Client,
};
use primitive_types::H256;
use subxt_signer::sr25519::Keypair;

pub fn alice() -> Keypair {
	subxt_signer::sr25519::dev::alice()
}

pub fn bob() -> Keypair {
	subxt_signer::sr25519::dev::bob()
}

pub fn charlie() -> Keypair {
	subxt_signer::sr25519::dev::charlie()
}

pub fn dave() -> Keypair {
	subxt_signer::sr25519::dev::dave()
}

pub fn eve() -> Keypair {
	subxt_signer::sr25519::dev::eve()
}

pub fn ferdie() -> Keypair {
	subxt_signer::sr25519::dev::ferdie()
}

pub async fn nonce_state(client: &Client, address: &str, block_hash: Option<H256>) -> Result<u32, ClientError> {
	let account = account_id_from_str(address)?;
	let block_hash = match block_hash {
		Some(x) => x,
		None => rpc::chain::get_block_hash(client, None).await?,
	};
	let block = client.online_client.blocks().at(block_hash).await?;

	Ok(block.account_nonce(&account).await? as u32)
}

pub async fn nonce_node(client: &Client, address: &str) -> Result<u32, ClientError> {
	let account = account_id_from_str(address)?;
	let nonce = rpc::system::account_next_index(client, account.to_string()).await;
	nonce.map_err(ClientError::from)
}

pub async fn nonce(client: &Client, address: &str) -> Result<u32, ClientError> {
	nonce_node(client, address).await
}

pub async fn app_keys(client: &Client, account_id: AccountId) -> Result<Vec<(String, u32)>, String> {
	let block_hash = rpc::chain::get_block_hash(client, None).await;
	let block_hash = block_hash.map_err(|e| e.to_string())?;

	let storage = client.storage().at(block_hash);
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

pub async fn app_ids(client: &Client, account_id: AccountId) -> Result<Vec<u32>, String> {
	let keys = match app_keys(client, account_id).await {
		Ok(k) => k,
		Err(e) => return Err(e),
	};

	Ok(keys.into_iter().map(|v| v.1).collect())
}

pub async fn account_info(client: &Client, account_id: AccountId) -> Result<Account, String> {
	let block_hash = rpc::chain::get_block_hash(client, None).await;
	let block_hash = block_hash.map_err(|e| e.to_string())?;

	let storage = client.storage().at(block_hash);
	let address = avail::storage().system().account(account_id);
	let result = storage.fetch_or_default(&address).await.map_err(|e| e.to_string())?;

	Ok(result)
}

pub fn account_id_from_str(value: &str) -> Result<AccountId, String> {
	value.parse().map_err(|e| std::format!("{:?}", e))
}

pub fn account_id_from_slice(value: &[u8]) -> Result<AccountId, String> {
	dbg!(&value);
	let account_id: [u8; 32] = match value.try_into() {
		Ok(x) => x,
		Err(err) => return Err(err.to_string()),
	};

	Ok(AccountId { 0: account_id })
}
