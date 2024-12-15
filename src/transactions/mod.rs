pub mod balances;
pub mod da;
pub mod nom_pools;
pub mod session;
pub mod staking;
pub mod vector;

use crate::AOnlineClient;
use subxt::backend::rpc::RpcClient;

pub use crate::avail::{
	balances::events as BalancesEvents, data_availability::events as DataAvailabilityEvents,
	nomination_pools::events as NominationPoolsEvents, session::events as SessionEvents,
	staking::events as StakingEvents, system::events as SystemEvents,
};

pub use crate::avail::{
	balances::calls::types as BalancesCalls,
	data_availability::calls::types as DataAvailabilityCalls,
	nomination_pools::calls::types as NominationPoolsCalls, session::calls::types as SessionCalls,
	staking::calls::types as StakingCalls, system::calls::types as SystemCalls,
};

#[derive(Clone)]
pub struct Transactions {
	pub balances: balances::Balances,
	pub staking: staking::Staking,
	pub data_availability: da::DataAvailability,
	pub session: session::Session,
	pub nomination_pools: nom_pools::NominationPools,
}

impl Transactions {
	pub fn new(online_client: AOnlineClient, rpc_client: RpcClient) -> Self {
		Self {
			balances: balances::Balances::new(online_client.clone(), rpc_client.clone()),
			staking: staking::Staking::new(online_client.clone(), rpc_client.clone()),
			data_availability: da::DataAvailability::new(online_client.clone(), rpc_client.clone()),
			session: session::Session::new(online_client.clone(), rpc_client.clone()),
			nomination_pools: nom_pools::NominationPools::new(
				online_client.clone(),
				rpc_client.clone(),
			),
		}
	}
}
