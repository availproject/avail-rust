mod api_dev;
mod from_substrate;

pub mod client;
pub mod config;
pub mod constants;
pub mod error;
pub mod extensions;
pub mod platform;
pub mod primitives;
pub mod transaction;
pub mod transaction_options;
pub mod transactions;
pub mod utils;

pub use api_dev::api as avail;
pub use avail::runtime_types::{bounded_collections::bounded_vec::BoundedVec, sp_arithmetic::per_things::Perbill};
pub use primitives::{
	block::{AppUncheckedExtrinsic, AvailHeader, DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder},
	kate::{Cell, GDataProof, GRow},
};
pub use transaction::{
	BlockState, ReceiptMethod, SubmittableTransaction, SubmittedTransaction, TransactionExtra, TransactionReceipt,
};

// External
pub mod ext {
	pub use codec;
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
	pub use super::config::*;
	pub use super::constants::*;
	pub use super::ext::*;
	pub use super::extensions::*;
	pub use super::transaction_options::*;
	pub use primitive_types::{H256, U256};
	pub use subxt_signer::{sr25519::Keypair, SecretUri};

	pub use super::{
		avail, client::Client, error::ClientError, error::RpcError, BlockState, BoundedVec, Perbill, ReceiptMethod,
		SubmittableTransaction, SubmittedTransaction, TransactionExtra, TransactionReceipt,
	};
}
