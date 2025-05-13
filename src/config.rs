pub use crate::primitives;
use crate::{DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder};
use subxt_core::config::substrate::BlakeTwo256;
use subxt_core::Config;
use subxt_rpcs::methods::legacy::BlockDetails as BlockDetailsRPC;

#[cfg(not(feature = "subxt"))]
use subxt_rpcs::RpcConfig;

#[cfg(feature = "subxt")]
pub use subxt_types::*;

/// Clients
#[cfg(feature = "subxt")]
pub mod subxt_types {
	use super::AvailConfig;
	use subxt::{
		blocks::{Block, BlocksClient, ExtrinsicDetails, ExtrinsicEvents, Extrinsics, FoundExtrinsic},
		constants::ConstantsClient,
		events::{EventDetails, Events, EventsClient},
		storage::StorageClient,
		tx::TxClient,
		OnlineClient,
	};

	pub type AOnlineClient = OnlineClient<AvailConfig>;
	pub type ABlocksClient = BlocksClient<AvailConfig, AOnlineClient>;
	pub type AStorageClient = StorageClient<AvailConfig, AOnlineClient>;
	pub type AConstantsClient = ConstantsClient<AvailConfig, AOnlineClient>;
	pub type AEventsClient = EventsClient<AvailConfig, AOnlineClient>;
	pub type ATxClient = TxClient<AvailConfig, AOnlineClient>;

	/// TX status
	pub type AExtrinsicEvents = ExtrinsicEvents<AvailConfig>;
	pub type AEvents = Events<AvailConfig>;
	pub type AEventDetails = EventDetails<AvailConfig>;
	pub type AExtrinsicDetails = ExtrinsicDetails<AvailConfig, AOnlineClient>;
	pub type AFoundExtrinsic<T> = FoundExtrinsic<AvailConfig, AOnlineClient, T>;
	pub type AExtrinsics = Extrinsics<AvailConfig, AOnlineClient>;
	pub type ABlock = Block<AvailConfig, AOnlineClient>;
}

/// Used only when chain_getBlock RPC is called. This is part of legacy baggage.
pub type ABlockDetailsRPC = BlockDetailsRPC<AvailConfig>;

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction for a avail node.
pub type AvailExtrinsicParams<T> = DefaultExtrinsicParams<T>;

/// A builder which leads to [`PolkadotExtrinsicParams`] being constructed.
/// This is what you provide to methods like `sign_and_submit()`.
pub type AvailExtrinsicParamsBuilder = DefaultExtrinsicParamsBuilder<AvailConfig>;

#[derive(Clone, Debug, Default)]
pub struct AvailConfig;

impl Config for AvailConfig {
	type AccountId = primitives::AccountId;
	type Address = primitives::MultiAddress;
	type ExtrinsicParams = AvailExtrinsicParams<Self>;
	type Hash = primitives::BlockHash;
	type Hasher = BlakeTwo256;
	type Header = primitives::AvailHeader;
	type Signature = primitives::MultiSignature;
	type AssetId = u32;
}

#[cfg(not(feature = "subxt"))]
impl RpcConfig for AvailConfig {
	type Header = primitives::AvailHeader;
	type Hash = primitives::BlockHash;
	type AccountId = primitives::AccountId;
}
