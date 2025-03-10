mod api_dev;
mod config;
mod from_substrate;

#[cfg(feature = "native")]
pub mod http;

// Export types for internal and external consumption
pub mod account;
pub mod block;
pub mod block_transaction;
pub mod da_commitments;
pub mod error;
pub mod primitives;
pub mod rpc;
pub mod runtime_api;
pub mod sdk;
pub mod transaction;
pub mod transactions;
pub mod utils;

pub use api_dev::api as avail;
pub use avail::runtime_types::{bounded_collections::bounded_vec::BoundedVec, sp_arithmetic::per_things::Perbill};
pub use avail_core;
pub use block::Block;
pub use block_transaction::{BlockTransaction, Filter};
pub use config::*;
pub use hex;
pub use kate_recovery;
pub use primitive_types::H256;
pub use primitives::{
	block::{AppUncheckedExtrinsic, AvailHeader, DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder},
	kate::{Cell, GDataProof, GRow},
};
pub use rpc::TransactionState;
pub use sdk::{Client, WaitFor, SDK};
pub use sp_core;
pub use subxt::{self, config::polkadot::U256};
pub use subxt_signer::{self, sr25519::Keypair, SecretUri};
pub use transaction::{Options, PopulatedOptions, Transaction, TransactionDetails};

pub mod prelude {
	pub use super::{
		account,
		account::account_id_from_str,
		avail,
		avail::runtime_types::bounded_collections::bounded_vec::BoundedVec,
		avail_core,
		block::{to_ascii, DataSubmission},
		config::*,
		error::ClientError,
		hex, kate_recovery, primitives, rpc,
		sdk::ClientMode,
		subxt, subxt_signer,
		transaction::watcher::Watcher,
		utils::new_h256_from_hex,
		AccountId, Block, BlockTransaction, Client, Filter, Keypair, MultiAddress, Options, Perbill, PopulatedOptions,
		SecretUri, Transaction, TransactionDetails, TransactionState, WaitFor, H256, SDK, U256,
	};
}
