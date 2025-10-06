pub mod author;
#[cfg(feature = "next")]
pub mod blob;
pub mod chain;
pub mod chainspec;
pub mod error;
pub mod grandpa;
pub mod kate;
pub mod rpc_methods;
pub mod runtime_api;
pub mod state;
pub mod system;

pub use error::Error;

pub use super::AvailHeader;
pub use chain::{Block, BlockJustification, LegacyBlock};
use subxt_rpcs::{RpcClient, client::RpcParams};
pub use system::{
	fetch_events::{BlockPhaseEvent, Filter as EventFilter, Options as EventOpts, PhaseEvent},
	fetch_extrinsics::{EncodeSelector, ExtrinsicFilter, ExtrinsicInfo, Options as ExtrinsicOpts, SignerPayload},
};

pub async fn raw_call<T: serde::de::DeserializeOwned>(
	client: &RpcClient,
	method: &str,
	params: RpcParams,
) -> Result<T, Error> {
	let value = client.request(method, params).await?;
	Ok(value)
}
