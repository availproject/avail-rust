pub mod clients;
pub mod config;
pub mod constants;
pub mod error;
pub mod extensions;
pub mod platform;
pub mod subscription;
pub mod transaction;
pub mod transaction_options;
pub mod transactions;

pub use clients::Client;
pub use constants::{
	LOCAL_ENDPOINT, LOCAL_WS_ENDPOINT, MAINNET_ENDPOINT, MAINNET_WS_ENDPOINT, TURING_ENDPOINT, TURING_WS_ENDPOINT,
};
pub use error::ClientError;
pub use extensions::{AccountIdExt, H256Ext, KeypairExt, SecretUriExt};
pub use transaction::{
	BlockState, ReceiptMethod, SubmittableTransaction, SubmittableTransactionLike, SubmittedTransaction,
	TransactionReceipt,
};
pub use transaction_options::{MortalityOption, Options, RefinedMortality, RefinedOptions};

#[cfg(feature = "subxt")]
pub use avail_rust_core::ext::subxt;
pub use avail_rust_core::{
	self, FetchEventsV1Options, FetchExtrinsicsV1Options,
	ext::{codec, primitive_types, scale_info, subxt_core, subxt_rpcs, subxt_signer},
};

#[cfg(feature = "generated_metadata")]
pub use avail_rust_core::avail_generated;

#[cfg(feature = "generated_metadata")]
pub use extensions::DefaultPayloadExt;

// Exporting types from ext libraries
pub use avail_rust_core::{
	AccountId, AvailHeader, BlockLocation, CompactDataLookup, DecodedTransaction, EncodeSelector, HasEventEmittedIndex,
	HasTxDispatchIndex, HashNumber, HeaderExtension, KateCommitment, MultiAddress, OpaqueTransaction, StorageDoubleMap,
	StorageDoubleMapIterator, StorageHasher, StorageMap, StorageMapIterator, StorageValue, Transaction,
	TransactionAdditional, TransactionCall, TransactionConvertible, TransactionDecodable, TransactionEventDecodable,
	TransactionEventEncodable, TransactionExtra, TransactionPayload, TransactionSigned, V3HeaderExtension, avail,
};
pub use primitive_types::{H256, U256};
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
