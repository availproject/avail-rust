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
//!     let client = Client::new(TURING_ENDPOINT).await?;
//!     let best = client.best().block_header().await?;
//!     println!("Best block: {:?}", best.hash());
//!     Ok(())
//! }
//! ```

pub mod block;
pub mod chain;
pub mod client;
pub mod clients;
pub mod config;
pub mod constants;
pub mod conversions;
pub mod error;
pub mod extensions;
pub mod platform;
pub mod submission;
pub mod subscription;
pub mod transaction_api;
pub mod transaction_options;
pub mod utils;

pub use client::Client;
pub use constants::{
	LOCAL_ENDPOINT, LOCAL_WS_ENDPOINT, MAINNET_ENDPOINT, MAINNET_WS_ENDPOINT, ONE_AVAIL, ONE_HUNDRED_AVAIL,
	ONE_THOUSAND_AVAIL, TEN_AVAIL, TURING_ENDPOINT, TURING_WS_ENDPOINT,
};
pub use extensions::{AccountIdExt, H256Ext, KeypairExt, SecretUriExt};
pub use submission::{BlockState, SubmittableTransaction, SubmittedTransaction, TransactionReceipt};
pub use transaction_options::{MortalityOption, Options, RefinedMortality, RefinedOptions};

pub use avail_rust_core::{
	self, AccountId, AvailHeader, BlockInfo, CompactDataLookup, EncodeSelector, EncodedExtrinsic, Extrinsic,
	ExtrinsicCall, ExtrinsicDecodable, ExtrinsicExtra, ExtrinsicSignature, HasHeader, HashNumber, HeaderExtension,
	KateCommitment, MultiAddress, RpcError, StorageDoubleMap, StorageDoubleMapIterator, StorageHasher, StorageMap,
	StorageMapIterator, StorageValue, TransactionEventDecodable, TransactionEventEncodable, V3HeaderExtension, avail,
	ext::{codec, primitive_types, scale_info, subxt_core, subxt_rpcs, subxt_signer},
	grandpa::GrandpaJustification,
	multi_account_id,
	rpc::LegacyBlock,
};
pub use constants::dev_accounts;
pub use error::{Error, UserError};
pub use primitive_types::{H256, U256};
pub use subscription::Sub;
pub use subxt_signer::{SecretUri, sr25519::Keypair};

// External
pub mod ext {
	pub use avail_rust_core::{self, ext::*};

	#[cfg(feature = "reqwest")]
	pub use reqwest;
}

pub mod prelude {
	pub use super::{config::*, constants::dev_accounts::*, *};
}
