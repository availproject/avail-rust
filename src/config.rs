pub use crate::primitives;
pub use crate::primitives::config::*;
use crate::{AvailHeader, DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder};
use codec::{Decode, Encode};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use subxt_core::config::substrate::BlakeTwo256;
use subxt_core::Config;
use subxt_rpcs::methods::legacy::BlockDetails as BlockDetailsRPC;

#[cfg(not(feature = "subxt"))]
use subxt_rpcs::RpcConfig;

#[cfg(feature = "subxt")]
pub use subxt_types::*;

/// Chain Primitives
pub type Signature = MultiSignature;

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
	type Header = AvailHeader;
	type Signature = primitives::MultiSignature;
	type AssetId = u32;
}

#[cfg(not(feature = "subxt"))]
impl RpcConfig for AvailConfig {
	type Header = AvailHeader;
	type Hash = primitives::BlockHash;
	type AccountId = primitives::AccountId;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlockId {
	pub hash: H256,
	pub height: u32,
}

impl From<(H256, u32)> for BlockId {
	fn from(value: (H256, u32)) -> Self {
		Self {
			hash: value.0,
			height: value.1,
		}
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransactionLocation {
	pub hash: H256,
	pub index: u32,
}

impl From<(H256, u32)> for TransactionLocation {
	fn from(value: (H256, u32)) -> Self {
		Self {
			hash: value.0,
			index: value.1,
		}
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HashIndex {
	Hash(H256),
	Index(u32),
}

/// A phase of a block's execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum RuntimePhase {
	/// Applying an extrinsic.
	ApplyExtrinsic(u32),
	/// Finalizing the block.
	Finalization,
	/// Initializing the block.
	Initialization,
}

pub type DispatchIndex = (u8, u8);
pub type EmittedIndex = (u8, u8);

#[cfg(not(feature = "subxt_metadata"))]
pub mod no_subxt_metadata {
	use codec::{Decode, Encode};
	use scale_decode::DecodeAsType;
	use scale_encode::EncodeAsType;

	#[derive(Debug, Clone, Encode, Decode, DecodeAsType, EncodeAsType)]
	pub struct AccountData {
		pub free: u128,
		pub reserved: u128,
		pub frozen: u128,
		pub flags: u128,
	}

	#[derive(Debug, Clone, Encode, Decode, DecodeAsType, EncodeAsType)]
	pub struct AccountInfo {
		pub nonce: u32,
		pub consumers: u32,
		pub providers: u32,
		pub sufficients: u32,
		pub data: AccountData,
	}
}

#[cfg(not(feature = "subxt_metadata"))]
pub use no_subxt_metadata::*;

#[cfg(feature = "subxt_metadata")]
pub type AccountData = crate::avail::runtime_types::pallet_balances::types::AccountData<u128>;
#[cfg(feature = "subxt_metadata")]
pub use crate::avail::system::storage::types::account::Account as AccountInfo;
