#[cfg(feature = "subxt_metadata")]
mod api_dev;
pub mod api_dev_custom;

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
pub use api_dev::api as subxt_avail;
pub use api_dev_custom as avail;

/* #[cfg(feature = "subxt_metadata")]
pub use avail::runtime_types::sp_arithmetic::per_things::Perbill; */

pub use bounded_collections::BoundedVec;
pub use primitive_types::H256;
pub use primitives::{
	kate::{Cell, GDataProof, GRow},
	AvailHeader, DecodedTransaction, DefaultExtrinsicParams,
};
pub use transaction::{
	BlockState, ReceiptMethod, SubmittableTransaction, SubmittableTransactionLike, SubmittedTransaction,
	TransactionReceipt,
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
	pub use super::*;

	pub use super::config::*;
	pub use super::constants::dev_accounts::*;
	pub use super::constants::*;
	pub use super::ext::*;
	pub use super::extensions::*;
	pub use super::transaction_options::*;
	pub use primitive_types::{H256, U256};
	pub use primitives::AccountId;
	pub use subxt_signer::{sr25519::Keypair, SecretUri};

	pub use super::{
		client::Client, error::ClientError, error::RpcError, BlockState, BoundedVec, ReceiptMethod,
		SubmittableTransaction, SubmittedTransaction, TransactionReceipt,
	};
}
