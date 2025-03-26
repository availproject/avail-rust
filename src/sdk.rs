use crate::{client::Client, error::ClientError, transactions::Transactions};
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaitFor {
	BlockInclusion,
	BlockFinalization,
}

impl WaitFor {
	pub fn to_str(&self) -> &'static str {
		match self {
			WaitFor::BlockInclusion => "Block Inclusion",
			WaitFor::BlockFinalization => "Block Finalization",
		}
	}
}
