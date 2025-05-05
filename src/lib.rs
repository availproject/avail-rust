mod api_dev;
mod config;
mod from_substrate;

// Export types for internal and external consumption
pub mod account;
pub mod client;
pub mod constants;
pub mod error;
pub mod extensions;
pub mod platform;
pub mod primitives;
pub mod transaction;
pub mod transactions;
pub mod utils;

pub use api_dev::api as avail;
pub use avail::runtime_types::{bounded_collections::bounded_vec::BoundedVec, sp_arithmetic::per_things::Perbill};
pub use client::Client;
pub use config::*;
pub use primitive_types::{H256, U256};
pub use primitives::{
	block::{AppUncheckedExtrinsic, AvailHeader, DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder},
	kate::{Cell, GDataProof, GRow},
};
pub use subxt_signer::{sr25519::Keypair, SecretUri};
pub use transaction::{
	BlockId, BlockState, MortalityOption, Options, PopulatedOptions, ReceiptMethod, SubmittableTransaction,
	SubmittedTransaction, TransactionExtra, TransactionLocation, TransactionReceipt,
};

// External
pub mod ext {
	pub use hex;
	pub use primitive_types;
	pub use scale_info;
	pub use serde;
	pub use serde_json;
	pub use subxt;
	pub use subxt_core;
	pub use subxt_rpcs;
	pub use subxt_signer;
}

pub mod prelude {
	pub use super::ext::*;
	pub use super::extensions::*;

	pub use super::{
		account, avail, config::*, error::ClientError, error::RpcError, primitives, AccountId, BlockId, BlockState,
		BoundedVec, Client, Keypair, MortalityOption, MultiAddress, Options, Perbill, PopulatedOptions, ReceiptMethod,
		SecretUri, SubmittableTransaction, SubmittedTransaction, TransactionExtra, TransactionLocation,
		TransactionReceipt,
	};
}
