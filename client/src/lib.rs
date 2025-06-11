pub mod clients;
pub mod config;
pub mod constants;
pub mod error;
pub mod extensions;
pub mod platform;
pub mod transaction;
pub mod transaction_options;
pub mod transactions;
pub mod utils;

pub use bounded_collections::BoundedVec;
pub use clients::Client;
pub use transaction::{
	BlockState, ReceiptMethod, SubmittableTransaction, SubmittableTransactionLike, SubmittedTransaction,
	TransactionReceipt,
};

#[cfg(feature = "subxt")]
use avail_rust_core::ext::subxt;
use avail_rust_core::{
	avail,
	ext::{codec, primitive_types, subxt_core, subxt_rpcs, subxt_signer},
};

// External
pub mod ext {
	pub use avail_rust_core::{self, ext::*};

	#[cfg(feature = "reqwest")]
	pub use reqwest;
}

pub mod prelude {
	pub use super::{
		config::*,
		constants::{dev_accounts::*, *},
		ext,
		extensions::*,
		transaction_options::*,
		*,
	};
	pub use avail::{RuntimeCall, RuntimeEvent};
	#[cfg(feature = "generated_metadata")]
	pub use avail_rust_core::avail_generated;
	pub use avail_rust_core::{
		avail,
		decoded_transaction::{DecodedTransaction, OpaqueTransaction},
		transaction::Transaction,
		AccountId, MultiAddress,
	};
	pub use primitive_types::{H256, U256};
	pub use subxt_signer::{sr25519::Keypair, SecretUri};

	pub use super::{
		clients::Client, error::ClientError, BlockState, BoundedVec, ReceiptMethod, SubmittableTransaction,
		SubmittableTransactionLike, SubmittedTransaction, TransactionReceipt,
	};
}
