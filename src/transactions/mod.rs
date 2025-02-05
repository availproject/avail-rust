pub mod balances;
pub mod da;
pub mod nom_pools;
pub mod proxy;
pub mod session;
pub mod staking;
pub mod vector;

pub use crate::avail::{
	balances::events as BalancesEvents, data_availability::events as DataAvailabilityEvents,
	nomination_pools::events as NominationPoolsEvents, session::events as SessionEvents,
	staking::events as StakingEvents, system::events as SystemEvents,
};

pub use crate::avail::{
	balances::calls::types as BalancesCalls, data_availability::calls::types as DataAvailabilityCalls,
	nomination_pools::calls::types as NominationPoolsCalls, session::calls::types as SessionCalls,
	staking::calls::types as StakingCalls, system::calls::types as SystemCalls,
};
use crate::Client;

#[derive(Clone)]
pub struct Transactions {
	pub balances: balances::Balances,
	pub staking: staking::Staking,
	pub data_availability: da::DataAvailability,
	pub session: session::Session,
	pub nomination_pools: nom_pools::NominationPools,
	pub proxy: proxy::Proxy,
}

impl Transactions {
	pub fn new(client: Client) -> Self {
		Self {
			balances: balances::Balances { client: client.clone() },
			staking: staking::Staking { client: client.clone() },
			data_availability: da::DataAvailability { client: client.clone() },
			session: session::Session { client: client.clone() },
			nomination_pools: nom_pools::NominationPools { client: client.clone() },
			proxy: proxy::Proxy { client: client.clone() },
		}
	}
}
