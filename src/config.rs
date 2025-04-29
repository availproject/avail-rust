use crate::{AvailHeader, DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder};
use subxt::{
	backend::legacy::rpc_methods::BlockDetails as BlockDetailsRPC,
	blocks::{
		Block, BlocksClient, ExtrinsicDetails, ExtrinsicEvents, ExtrinsicSignedExtensions, Extrinsics, FoundExtrinsic,
	},
	config::substrate::BlakeTwo256,
	constants::ConstantsClient,
	events::{EventDetails, Events, EventsClient},
	storage::StorageClient,
	tx::{TxClient, TxInBlock, TxProgress},
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
pub type AExtrinsicSignedExtensions<'a> = ExtrinsicSignedExtensions<'a, AvailConfig>;

/// TX status
pub type ATxProgress = TxProgress<AvailConfig, AOnlineClient>;
pub type ATxInBlock = TxInBlock<AvailConfig, AOnlineClient>;
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

#[derive(Clone, Copy, Default, Debug)]
pub struct AppId(pub avail_core::AppId);

impl From<avail_core::AppId> for AppId {
	fn from(value: avail_core::AppId) -> Self {
		Self(value)
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
