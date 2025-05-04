use crate::{AvailHeader, DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder};
use codec::{Compact, Decode, Encode};
use subxt::{
	backend::legacy::rpc_methods::BlockDetails as BlockDetailsRPC,
	blocks::{Block, BlocksClient, ExtrinsicDetails, ExtrinsicEvents, Extrinsics, FoundExtrinsic},
	config::substrate::BlakeTwo256,
	constants::ConstantsClient,
	events::{EventDetails, Events, EventsClient},
	storage::StorageClient,
	tx::TxClient,
	utils::{AccountId32, MultiSignature, H256},
	Config, OnlineClient,
};

/// Chain Primitives
pub type AccountId = AccountId32;
pub type AccountIndex = u32;
pub type MultiAddress = subxt::utils::MultiAddress<AccountId, AccountIndex>;
pub type Signature = MultiSignature;
pub type BlockNumber = u32;
pub type BlockHash = H256;

/// Clients
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

/// Used only when chain_getBlock RPC is called. This is part of legacy baggage.
pub type ABlockDetailsRPC = BlockDetailsRPC<AvailConfig>;

/// A struct representing the signed extra and additional parameters required
/// to construct a transaction for a avail node.
pub type AvailExtrinsicParams<T> = DefaultExtrinsicParams<T>;

/// A builder which leads to [`PolkadotExtrinsicParams`] being constructed.
/// This is what you provide to methods like `sign_and_submit()`.
pub type AvailExtrinsicParamsBuilder = DefaultExtrinsicParamsBuilder<AvailConfig>;

#[derive(Clone, Copy, Debug, Encode, Decode, Eq, PartialEq)]
pub struct AppId(pub Compact<u32>);

impl Default for AppId {
	fn default() -> Self {
		Self(Compact(0))
	}
}

impl From<u32> for AppId {
	fn from(value: u32) -> Self {
		Self(Compact(value))
	}
}

#[derive(Clone, Debug, Default)]
pub struct AvailConfig;

impl Config for AvailConfig {
	type AccountId = AccountId;
	type Address = MultiAddress;
	type ExtrinsicParams = AvailExtrinsicParams<Self>;
	type Hash = BlockHash;
	type Hasher = BlakeTwo256;
	type Header = AvailHeader;
	type Signature = Signature;
	type AssetId = u32;
}
