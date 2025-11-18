/// One AVAIL in base units.
pub const ONE_AVAIL: u128 = 1_000_000_000_000_000_000u128;
/// Ten AVAIL in base units.
pub const TEN_AVAIL: u128 = 10_000_000_000_000_000_000u128;
/// One hundred AVAIL in base units.
pub const ONE_HUNDRED_AVAIL: u128 = 100_000_000_000_000_000_000u128;
/// One thousand AVAIL in base units.
pub const ONE_THOUSAND_AVAIL: u128 = 1_000_000_000_000_000_000_000u128;
/// Default HTTP endpoint for a locally running Avail node.
pub const LOCAL_ENDPOINT: &str = "http://127.0.0.1:9944";
/// Default WebSocket endpoint for a locally running Avail node.
pub const LOCAL_WS_ENDPOINT: &str = "ws://127.0.0.1:9944";
/// Public HTTP endpoint for the Turing test network.
pub const TURING_ENDPOINT: &str = "https://turing-rpc.avail.so/rpc";
/// Public WebSocket endpoint for the Turing test network.
pub const TURING_WS_ENDPOINT: &str = "wss://turing-rpc.avail.so/ws";
/// Public HTTP endpoint for the Avail mainnet.
pub const MAINNET_ENDPOINT: &str = "https://mainnet-rpc.avail.so/rpc";
/// Public WebSocket endpoint for the Avail mainnet.
pub const MAINNET_WS_ENDPOINT: &str = "wss://mainnet-rpc.avail.so/ws";

/// Development accounts useful for testing and examples.
pub mod dev_accounts {
	/// Re-exports development keypairs for local testing.
	pub use crate::subxt_signer::sr25519::dev::*;
}
