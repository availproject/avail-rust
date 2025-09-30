pub mod block_api;
pub mod clients;
pub mod config;
pub mod constants;
pub mod error;
pub mod extensions;
pub mod platform;
pub mod submission_api;
pub mod subscription;
pub mod transaction_api;
pub mod transaction_options;

pub use block_api::{ExtrinsicEvent, ExtrinsicEvents};
pub use clients::Client;
pub use constants::{
	LOCAL_ENDPOINT, LOCAL_WS_ENDPOINT, MAINNET_ENDPOINT, MAINNET_WS_ENDPOINT, ONE_AVAIL, ONE_HUNDRED_AVAIL,
	ONE_THOUSAND_AVAIL, TEN_AVAIL, TURING_ENDPOINT, TURING_WS_ENDPOINT,
};
pub use extensions::{AccountIdExt, H256Ext, KeypairExt, SecretUriExt};
pub use submission_api::{BlockState, SubmittableTransaction, SubmittedTransaction, TransactionReceipt};
pub use transaction_options::{MortalityOption, Options, RefinedMortality, RefinedOptions};

pub use avail_rust_core::{
	self, AccountId, AvailHeader, BlockInfo, CompactDataLookup, EncodeSelector, Extrinsic, ExtrinsicCall,
	ExtrinsicExtra, ExtrinsicSignature, HasHeader, HashNumber, HeaderExtension, KateCommitment, MultiAddress,
	RawExtrinsic, RpcError, StorageDoubleMap, StorageDoubleMapIterator, StorageHasher, StorageMap, StorageMapIterator,
	StorageValue, TransactionDecodable, TransactionEventDecodable, TransactionEventEncodable, V3HeaderExtension, avail,
	ext::{codec, primitive_types, scale_info, subxt_core, subxt_rpcs, subxt_signer},
	grandpa::GrandpaJustification,
	multi_account_id,
	rpc::LegacyBlock,
};
pub use block_api::{
	BlockApi, BlockEvents, BlockExtrinsic, BlockRawExtrinsic, BlockTransaction, BlockWithExt, BlockWithRawExt,
	BlockWithTx,
};
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
