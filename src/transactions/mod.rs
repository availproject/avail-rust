pub mod da;

#[cfg(feature = "subxt_metadata")]
pub mod balances;
#[cfg(feature = "subxt_metadata")]
pub mod nom_pools;
#[cfg(feature = "subxt_metadata")]
pub mod proxy;
#[cfg(feature = "subxt_metadata")]
pub mod session;
#[cfg(feature = "subxt_metadata")]
pub mod staking;
#[cfg(feature = "subxt_metadata")]
pub mod vector;

#[cfg(feature = "subxt_metadata")]
pub use crate::avail::{
	balances::events as BalancesEvents, data_availability::events as DataAvailabilityEvents,
	nomination_pools::events as NominationPoolsEvents, session::events as SessionEvents,
	staking::events as StakingEvents, system::events as SystemEvents,
};

#[cfg(feature = "subxt_metadata")]
pub use crate::avail::{
	balances::calls::types as BalancesCalls, data_availability::calls::types as DataAvailabilityCalls,
	nomination_pools::calls::types as NominationPoolsCalls, session::calls::types as SessionCalls,
	staking::calls::types as StakingCalls, system::calls::types as SystemCalls,
};
use crate::client::Client;

#[derive(Clone)]
pub struct Transactions {
	pub data_availability: da::DataAvailability,
	#[cfg(feature = "subxt_metadata")]
	pub balances: balances::Balances,
	#[cfg(feature = "subxt_metadata")]
	pub staking: staking::Staking,
	#[cfg(feature = "subxt_metadata")]
	pub session: session::Session,
	#[cfg(feature = "subxt_metadata")]
	pub nomination_pools: nom_pools::NominationPools,
	#[cfg(feature = "subxt_metadata")]
	pub proxy: proxy::Proxy,
	#[cfg(feature = "subxt_metadata")]
	pub vector: vector::Vector,
}

impl Transactions {
	pub fn new(client: Client) -> Self {
		Self {
			data_availability: da::DataAvailability { client: client.clone() },
			#[cfg(feature = "subxt_metadata")]
			balances: balances::Balances { client: client.clone() },
			#[cfg(feature = "subxt_metadata")]
			staking: staking::Staking { client: client.clone() },
			#[cfg(feature = "subxt_metadata")]
			session: session::Session { client: client.clone() },
			#[cfg(feature = "subxt_metadata")]
			nomination_pools: nom_pools::NominationPools { client: client.clone() },
			#[cfg(feature = "subxt_metadata")]
			proxy: proxy::Proxy { client: client.clone() },
			#[cfg(feature = "subxt_metadata")]
			vector: vector::Vector { client: client.clone() },
		}
	}
}
