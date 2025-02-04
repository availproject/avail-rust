use crate::{avail, AccountId, Client, Transaction};

pub type TransferAllCall = avail::balances::calls::types::TransferAll;
pub type TransferAllowDeathCall = avail::balances::calls::types::TransferAllowDeath;
pub type TransferKeepAliveCall = avail::balances::calls::types::TransferKeepAlive;

#[derive(Clone)]
pub struct Balances {
	pub(crate) client: Client,
}

impl Balances {
	pub fn transfer_all(&self, dest: AccountId, keep_alive: bool) -> Transaction<TransferAllCall> {
		let payload = avail::tx().balances().transfer_all(dest.into(), keep_alive);
		Transaction::new(self.client.clone(), payload)
	}

	pub fn transfer_allow_death(&self, dest: AccountId, amount: u128) -> Transaction<TransferAllowDeathCall> {
		let payload = avail::tx().balances().transfer_allow_death(dest.into(), amount);
		Transaction::new(self.client.clone(), payload)
	}

	pub fn transfer_keep_alive(&self, dest: AccountId, value: u128) -> Transaction<TransferKeepAliveCall> {
		let payload = avail::tx().balances().transfer_keep_alive(dest.into(), value);
		Transaction::new(self.client.clone(), payload)
	}
}
