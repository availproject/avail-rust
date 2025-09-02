pub mod author;
pub mod chain;
pub mod chainspec;
pub mod grandpa;
pub mod kate;
pub mod rpc_methods;
pub mod state;
pub mod system;

pub use super::AvailHeader;
pub use chain::{Block, BlockJustification, BlockWithJustifications};
use subxt_rpcs::{RpcClient, client::RpcParams};
pub use system::fetch_extrinsics::EncodeSelector;

pub async fn call_raw<T: serde::de::DeserializeOwned>(
	client: &RpcClient,
	method: &str,
	params: RpcParams,
) -> Result<T, subxt_rpcs::Error> {
	let value = client.request(method, params).await?;
	Ok(value)
}
