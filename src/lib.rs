mod api_dev;
mod config;
mod from_substrate;

#[cfg(feature = "native")]
pub mod http;
#[cfg(feature = "native")]
pub mod turobda;
#[cfg(feature = "native")]
pub use turobda::TurboDA;

// Export types for internal and external consumption
pub mod account;
pub mod block;
pub mod block_transaction;
pub mod client;
pub mod client_rpc;
pub mod client_runtime_api;
pub mod error;
pub mod primitives;
pub mod sdk;
pub mod transaction;
pub mod transactions;
pub mod utils;

pub use api_dev::api as avail;
pub use avail::runtime_types::{bounded_collections::bounded_vec::BoundedVec, sp_arithmetic::per_things::Perbill};
pub use avail_core;
pub use block::Block;
pub use block_transaction::{BlockTransaction, Filter};
pub use client::{Client, ClientOptions};
pub use config::*;
pub use hex;
pub use kate_recovery;
pub use primitive_types;
pub use primitive_types::H256;
pub use primitives::{
	block::{AppUncheckedExtrinsic, AvailHeader, DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder},
	kate::{Cell, GDataProof, GRow},
};
pub use sdk::{AccountIdExt, H256Ext, SecretUriExt, SDK};
pub use subxt::{self, config::polkadot::U256};
pub use subxt_signer::{self, sr25519::Keypair, SecretUri};
pub use transaction::{
	BlockId, BlockState, Options, PopulatedOptions, ReceiptMethod, SubmittableTransaction, SubmittedTransaction,
	TransactionExtra, TransactionLocation, TransactionReceipt,
};

pub mod prelude {
	pub use super::{
		account, avail,
		avail::runtime_types::bounded_collections::bounded_vec::BoundedVec,
		avail_core,
		block::{to_ascii, DataSubmission},
		config::*,
		error::ClientError,
		hex, kate_recovery, primitives, subxt, subxt_signer, AccountId, AccountIdExt, Block, BlockId, BlockState,
		BlockTransaction, Client, ClientOptions, Filter, H256Ext, Keypair, MultiAddress, Options, Perbill,
		PopulatedOptions, ReceiptMethod, SecretUri, SecretUriExt, SubmittableTransaction, SubmittedTransaction,
		TransactionExtra, TransactionLocation, TransactionReceipt, H256, SDK, U256,
	};

	#[cfg(feature = "native")]
	pub use super::TurboDA;
}
