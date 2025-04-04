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
pub mod block;
pub mod block_transaction;
pub mod client;
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
pub use client::{Client, ClientMode, ClientOptions};
pub use config::*;
pub use hex;
pub use kate_recovery;
pub use poly_multiproof::{
	method1::{M1NoPrecomp, Proof},
	msm::blst::BlstMSMEngine,
	traits::PolyMultiProofNoPrecomp,
};
pub use primitive_types::H256;
pub use primitives::{
	block::{AppUncheckedExtrinsic, AvailHeader, DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder},
	kate::{Cell, GDataProof, GRow},
};
pub use rpc::TransactionState;
pub use sdk::{WaitFor, SDK};
pub use sp_core;
pub use subxt::{self, config::polkadot::U256};
pub use subxt_signer::{self, sr25519::Keypair, SecretUri};
pub use transaction::{
	watcher::{Watcher, WatcherMode, WatcherOptions},
	Options, PopulatedOptions, Transaction, TransactionDetails,
};

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
		hex, kate_recovery, primitives, rpc, subxt, subxt_signer,
		utils::new_h256_from_hex,
		AccountId, Block, BlockTransaction, Client, ClientMode, ClientOptions, Filter, Keypair, MultiAddress, Options,
		Perbill, PopulatedOptions, SecretUri, Transaction, TransactionDetails, TransactionState, WaitFor, Watcher,
		WatcherMode, WatcherOptions, H256, SDK, U256,
	};

	#[cfg(feature = "native")]
	pub use super::TurboDA;
}
