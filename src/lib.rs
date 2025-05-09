#[cfg(feature = "subxt_metadata")]
mod api_dev;
#[cfg(not(feature = "subxt_metadata"))]
mod api_dev_fake;

mod from_substrate;

pub mod block;
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

#[cfg(feature = "subxt_metadata")]
pub mod utils;

#[cfg(feature = "subxt_metadata")]
pub use api_dev::api as avail;
#[cfg(not(feature = "subxt_metadata"))]
pub use api_dev_fake::avail;

#[cfg(feature = "subxt_metadata")]
pub use avail::runtime_types::sp_arithmetic::per_things::Perbill;

pub use bounded_collections::BoundedVec;
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
	pub use subxt_core;
	pub use subxt_rpcs;
	pub use subxt_signer;

	#[cfg(feature = "subxt")]
	pub use subxt;
}

pub mod prelude {
	pub use super::config::*;
	pub use super::constants::*;
	pub use super::ext::*;
	pub use super::extensions::*;
	pub use super::transaction_options::*;
	pub use primitive_types::{H256, U256};
	pub use subxt_signer::{sr25519::Keypair, SecretUri};

	#[cfg(feature = "subxt_metadata")]
	pub use super::{avail, Perbill};

	pub use super::{
		client::Client, error::ClientError, error::RpcError, BlockState, BoundedVec, ReceiptMethod,
		SubmittableTransaction, SubmittedTransaction, TransactionExtra, TransactionReceipt,
	};
}
