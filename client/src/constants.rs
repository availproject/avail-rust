pub const ONE_AVAIL: u128 = 1_000_000_000_000_000_000u128;
pub const TEN_AVAIL: u128 = 10_000_000_000_000_000_000u128;
pub const ONE_HUNDRED_AVAIL: u128 = 100_000_000_000_000_000_000u128;
pub const ONE_THOUSAND_AVAIL: u128 = 1_000_000_000_000_000_000_000u128;
pub const LOCAL_ENDPOINT: &str = "http://127.0.0.1:9944";
pub const LOCAL_WS_ENDPOINT: &str = "ws://127.0.0.1:9944";
pub const TURING_ENDPOINT: &str = "https://turing-rpc.avail.so/rpc";
pub const TURING_WS_ENDPOINT: &str = "wss://turing-rpc.avail.so/ws";
pub const MAINNET_ENDPOINT: &str = "https://mainnet-rpc.avail.so/rpc";
pub const MAINNET_WS_ENDPOINT: &str = "wss://mainnet-rpc.avail.so/ws";

pub mod dev_accounts {
	pub use crate::subxt_signer::sr25519::dev::*;
}
