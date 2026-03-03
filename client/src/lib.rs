//! High-level Rust SDK for interacting with the Avail blockchain.
//!
//! This crate provides ergonomic helpers for connecting to Avail nodes, submitting transactions,
//! querying blocks and events, and subscribing to chain updates.
//!
//! # Quick Start
//!
//! ```no_run
//! use avail_rust_client::{Client, TURING_ENDPOINT};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::connect(TURING_ENDPOINT).await?;
//!     let best = client.best().block_header().await?;
//!     println!("Best block: {:?}", best.hash());
//!     Ok(())
//! }
//! ```

#[macro_use]
mod macros;

pub mod account;
pub mod block;
pub mod chain;
pub mod client;
pub mod clients;
pub mod config;
pub mod constants;
pub mod conversions;
pub mod error;
pub mod error_ops;
pub mod extensions;
pub mod platform;
pub mod retry_policy;
pub mod submission;
pub mod subscription;
pub mod transaction_api;
pub mod transaction_options;
pub mod utils;

pub use chain::{Head, HeadKind};
pub use client::Client;
#[cfg(feature = "reqwest")]
pub use client::ConnectionOptions;
#[cfg(feature = "tracing")]
pub use client::TracingFormat;
pub use constants::{
	LOCAL_ENDPOINT, LOCAL_WS_ENDPOINT, MAINNET_ENDPOINT, MAINNET_WS_ENDPOINT, ONE_AVAIL, ONE_HUNDRED_AVAIL,
	ONE_THOUSAND_AVAIL, TEN_AVAIL, THOUSAND_AVAIL, TURING_ENDPOINT, TURING_WS_ENDPOINT,
};
pub use extensions::{AccountIdExt, H256Ext};
pub use retry_policy::RetryPolicy;
pub use submission::{BlockState, SubmissionOutcome, SubmittableTransaction, SubmittedTransaction, TransactionReceipt};
pub use subscription::{BlockQueryMode, Fetcher, SubscribeApi, Subscription, SubscriptionBuilder, SubscriptionItem};
pub use transaction_options::{Mortality, MortalityOption, Options};

pub use account::Account;
pub use avail_rust_core::{
	self, AccountId, AvailHeader, BlockInfo, CompactDataLookup, DataFormat, Extension, ExtensionImplicit, Extrinsic,
	ExtrinsicCall, ExtrinsicDecodable, HasHeader, HashNumber, HeaderExtension, KateCommitment, MultiAddress, RpcError,
	TransactionEventDecodable, TransactionEventEncodable, V3HeaderExtension, avail,
	ext::{codec, primitive_types, scale_info, scale_value, subxt_core, subxt_metadata, subxt_rpcs, subxt_signer},
	grandpa::GrandpaJustification,
	multi_account_id,
	rpc::LegacyBlock,
	substrate::{
		StorageDoubleMap, StorageDoubleMapIterator, StorageHasher, StorageMap, StorageMapIterator, StorageValue,
	},
};
pub use constants::dev_accounts;
pub use error::{Error, ErrorCode, UserError};
pub use error_ops::*;
pub use primitive_types::{H256, U256};
pub use subscription::fetcher::{
	BlockEventsFetcher, BlockFetcher, BlockHeaderFetcher, BlockInfoFetcher, ExtrinsicFetcher,
	GrandpaJustificationFetcher, LegacyBlockFetcher, UntypedExtrinsicFetcher,
};
pub use subxt_signer::{SecretUri, sr25519::Keypair};

// External
pub mod ext {
	pub use async_trait::async_trait;
	pub use avail_rust_core::{self, ext::*};

	#[cfg(feature = "reqwest")]
	pub use reqwest;
}

pub mod prelude {
	pub use super::{config::*, constants::dev_accounts::*, *};
}
