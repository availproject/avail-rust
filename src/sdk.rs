use crate::{client::Client, error::ClientError, transactions::Transactions, AccountId, H256};
use std::fmt::Debug;

#[derive(Clone)]
pub struct SDK {
	pub client: Client,
	pub tx: Transactions,
}

impl SDK {
	pub async fn new(endpoint: &str) -> Result<Self, ClientError> {
		let client = super::client::reconnecting_api(endpoint).await?;

		Self::new_custom(client).await
	}

	pub async fn new_custom(client: Client) -> Result<Self, ClientError> {
		let tx = Transactions::new(client.clone());
		Ok(SDK { client, tx })
	}

	pub fn enable_logging() {
		env_logger::builder().init();
	}

	pub fn one_avail() -> u128 {
		1_000_000_000_000_000_000u128
	}

	pub fn local_endpoint() -> &'static str {
		"ws://127.0.0.1:9944"
	}

	pub fn local_http_endpoint() -> &'static str {
		"http://127.0.0.1:9944"
	}

	pub fn turing_endpoint() -> &'static str {
		"wss://turing-rpc.avail.so/ws"
	}

	pub fn turing_http_endpoint() -> &'static str {
		"https://turing-rpc.avail.so/rpc"
	}

	pub fn mainnet_endpoint() -> &'static str {
		"wss://mainnet-rpc.avail.so/ws"
	}

	pub fn mainnet_http_endpoint() -> &'static str {
		"https://mainnet-rpc.avail.so/rpc"
	}
}

#[cfg(feature = "native")]
impl SDK {
	pub async fn new_http(endpoint: &str) -> Result<Self, ClientError> {
		let client = super::client::http_api(endpoint).await?;

		Self::new_custom(client).await
	}
}

impl Debug for SDK {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let genesis_hash = self.client.online_client.genesis_hash();
		f.debug_struct("SDK")
			.field("Genesis Hash", &genesis_hash)
			.finish_non_exhaustive()
	}
}

pub trait H256Ext {
	fn from_hex(s: &str) -> Result<H256, String>;
}

impl H256Ext for H256 {
	fn from_hex(s: &str) -> Result<H256, String> {
		let mut s = s;
		if s.starts_with("0x") {
			s = &s[2..];
		}

		if s.len() != 64 {
			let msg = std::format!(
				"Failed to convert string to H256. Expected 64 bytes got {}. Input string: {}",
				s.len(),
				s
			);
			return Err(msg);
		}

		let block_hash = hex::decode(s).map_err(|e| e.to_string())?;
		let block_hash = TryInto::<[u8; 32]>::try_into(block_hash);
		match block_hash {
			Ok(v) => Ok(H256(v)),
			Err(e) => {
				let msg = std::format!("Failed to covert decoded string to H256. Input {:?}", e);
				Err(msg)
			},
		}
	}
}

pub trait AccountIdExt {
	fn from_str(value: &str) -> Result<AccountId, String>;
	fn from_slice(value: &[u8]) -> Result<AccountId, String>;
}

impl AccountIdExt for AccountId {
	fn from_str(value: &str) -> Result<AccountId, String> {
		value.parse().map_err(|e| std::format!("{:?}", e))
	}

	fn from_slice(value: &[u8]) -> Result<AccountId, String> {
		let account_id: [u8; 32] = match value.try_into() {
			Ok(x) => x,
			Err(err) => return Err(err.to_string()),
		};

		Ok(AccountId { 0: account_id })
	}
}
