mod api_dev;
mod config;
mod from_substrate;
mod sdk;

#[cfg(feature = "native")]
pub mod http;

// Export types for internal and external consumption
pub mod account;
pub mod block;
pub mod error;
pub mod primitives;
pub mod rpc;
pub mod transactions;
pub mod utils;

pub type RewardDestination =
	api_dev::api::runtime_types::pallet_staking::RewardDestination<AccountId>;

pub use api_dev::api::{
	data_availability::calls::types::{create_application_key::Key, submit_data::Data},
	runtime_types::{frame_support::dispatch::DispatchFeeModifier, pallet_staking::ValidatorPrefs},
};
pub use primitive_types::H256;
pub use subxt_signer::{sr25519::Keypair, SecretUri};

pub use api_dev::api as avail;
pub use config::*;
pub use sdk::{WaitFor, SDK};

pub use crate::avail::runtime_types::sp_arithmetic::per_things::Perbill;
pub use avail_core;
pub use block::Block;
pub use hex;
pub use kate_recovery;
pub use primitives::{
	block::{
		AppUncheckedExtrinsic, AvailHeader, DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder,
	},
	kate::{Cell, GDataProof, GRow},
};
pub use sp_core;
pub use subxt::{self, config::polkadot::U256};
pub use subxt_signer;
pub use transactions::{Mortality, Nonce, Options, PopulatedOptions};

pub mod nomination_pools_types {
	pub use crate::avail::nomination_pools::calls::types::{
		set_claim_permission::Permission, set_state::State,
	};
}
