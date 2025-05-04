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
pub use client::{Client, ClientOptions};
pub use config::*;
pub use hex;
pub use primitive_types::{self, H256};
pub use primitives::{
	block::{AppUncheckedExtrinsic, AvailHeader, DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder},
	kate::{Cell, GDataProof, GRow},
};
pub use sdk::{AccountIdExt, H256Ext, SecretUriExt, SDK};
pub use subxt::{self, config::polkadot::U256};
pub use subxt_signer::{self, sr25519::Keypair, SecretUri};
pub use transaction::{
	BlockId, BlockState, MortalityOption, Options, PopulatedOptions, ReceiptMethod, SubmittableTransaction,
	SubmittedTransaction, TransactionExtra, TransactionLocation, TransactionReceipt,
};

pub mod prelude {
	pub use super::{
		account, avail, avail::runtime_types::bounded_collections::bounded_vec::BoundedVec, config::*,
		error::ClientError, hex, primitives, subxt, subxt_signer, AccountId, AccountIdExt, BlockId, BlockState, Client,
		ClientOptions, H256Ext, Keypair, MortalityOption, MultiAddress, Options, Perbill, PopulatedOptions,
		ReceiptMethod, SecretUri, SecretUriExt, SubmittableTransaction, SubmittedTransaction, TransactionExtra,
		TransactionLocation, TransactionReceipt, H256, SDK, U256,
	};

	#[cfg(feature = "native")]
	pub use super::TurboDA;
}
