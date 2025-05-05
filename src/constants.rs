use subxt_signer::sr25519::Keypair;

pub const ONE_AVAIL: u128 = 1_000_000_000_000_000_000u128;
pub const LOCAL_ENDPOINT: &str = "http://127.0.0.1:9944";
pub const LOCAL_WS_ENDPOINT: &str = "ws://127.0.0.1:9944";
pub const TURING_ENDPOINT: &str = "https://turing-rpc.avail.so/rpc";
pub const TURING_WS_ENDPOINT: &str = "wss://turing-rpc.avail.so/ws";
pub const MAINNET_ENDPOINT: &str = "https://mainnet-rpc.avail.so/rpc";
pub const MAINNET_WS_ENDPOINT: &str = "wss://mainnet-rpc.avail.so/ws";

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
