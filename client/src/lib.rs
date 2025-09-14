pub mod block;
pub mod clients;
pub mod config;
pub mod constants;
pub mod error;
pub mod extensions;
pub mod extrinsic;
pub mod platform;
pub mod subscription;
pub mod transaction_options;
pub mod transactions;

pub use block::{ExtrinsicEvent, ExtrinsicEvents};
pub use clients::Client;
pub use constants::{
	LOCAL_ENDPOINT, LOCAL_WS_ENDPOINT, MAINNET_ENDPOINT, MAINNET_WS_ENDPOINT, TURING_ENDPOINT, TURING_WS_ENDPOINT,
};
pub(crate) use error::{Error, UserError};
pub use extensions::{AccountIdExt, H256Ext, KeypairExt, SecretUriExt};
pub use extrinsic::{BlockState, SubmittableTransaction, SubmittedTransaction, TransactionReceipt};
pub use transaction_options::{MortalityOption, Options, RefinedMortality, RefinedOptions};

pub use avail_rust_core::{
	self,
	ext::{codec, primitive_types, scale_info, subxt_core, subxt_rpcs, subxt_signer},
};

// Exporting types from ext libraries
pub use avail_rust_core::{
	AccountId, AvailHeader, BlockRef, CompactDataLookup, EncodeSelector, Extrinsic, ExtrinsicExtra, ExtrinsicSignature,
	HasHeader, HashNumber, HeaderExtension, KateCommitment, MultiAddress, RawExtrinsic, StorageDoubleMap,
	StorageDoubleMapIterator, StorageHasher, StorageMap, StorageMapIterator, StorageValue, TransactionDecodable,
	TransactionEventDecodable, TransactionEventEncodable, V3HeaderExtension, avail,
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
