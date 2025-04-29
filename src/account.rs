use crate::{
	avail::{self},
	error::ClientError,
	AccountId, Client,
};
use primitive_types::H256;
use std::str::FromStr;
use subxt_signer::{sr25519::Keypair, SecretUri};

pub fn from_secret_uri(uri: &str) -> Result<Keypair, ClientError> {
	let secret_uri = SecretUri::from_str(uri)?;
	let keypair = Keypair::from_uri(&secret_uri)?;
	Ok(keypair)
}

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

pub async fn app_keys(client: &Client, account_id: AccountId, block_hash: H256) -> Result<Vec<(String, u32)>, String> {
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

pub async fn app_ids(client: &Client, account_id: AccountId, block_hash: H256) -> Result<Vec<u32>, String> {
	let keys = match app_keys(client, account_id, block_hash).await {
		Ok(k) => k,
		Err(e) => return Err(e),
	};

	Ok(keys.into_iter().map(|v| v.1).collect())
}
