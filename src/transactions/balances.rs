use crate::{avail, AccountId, Client, Transaction};

pub type TransferAllCall = avail::balances::calls::types::TransferAll;
pub type TransferAllowDeathCall = avail::balances::calls::types::TransferAllowDeath;
pub type TransferKeepAliveCall = avail::balances::calls::types::TransferKeepAlive;

#[derive(Clone)]
pub struct Balances {
	pub(crate) client: Client,
}

impl Balances {
	/// Transfer the entire transferable balance from the caller account.
	///
	/// NOTE: This function only attempts to transfer _transferable_ balances. This means that
	/// any locked, reserved, or existential deposits (when `keep_alive` is `true`), will not be
	/// transferred by this function.
	pub fn transfer_all(&self, dest: AccountId, keep_alive: bool) -> Transaction<TransferAllCall> {
		let payload = avail::tx().balances().transfer_all(dest.into(), keep_alive);
		Transaction::new(self.client.clone(), payload)
	}

	/// Transfer some liquid free balance to another account.
	///
	/// `transfer_allow_death` will set the `FreeBalance` of the sender and receiver.
	/// If the sender's account is below the existential deposit as a result
	/// of the transfer, the account will be reaped.
	///
	/// The dispatch origin for this call must be `Signed` by the transactor.
	pub fn transfer_allow_death(&self, dest: AccountId, amount: u128) -> Transaction<TransferAllowDeathCall> {
		let payload = avail::tx().balances().transfer_allow_death(dest.into(), amount);
		Transaction::new(self.client.clone(), payload)
	}

	/// Same as the `TransferAlowDeath` call, but with a check that the transfer will not
	/// kill the origin account.
	pub fn transfer_keep_alive(&self, dest: AccountId, value: u128) -> Transaction<TransferKeepAliveCall> {
		let payload = avail::tx().balances().transfer_keep_alive(dest.into(), value);
		Transaction::new(self.client.clone(), payload)
	}
}
