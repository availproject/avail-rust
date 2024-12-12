use super::Transaction;
use crate::{avail, AOnlineClient};
use primitive_types::H256;
use subxt::backend::rpc::reconnecting_rpc_client::RpcClient;

pub type ExecuteCall = avail::vector::calls::types::Execute;
pub type FulfillCallCall = avail::vector::calls::types::FulfillCall;
pub type SendMessageCall = avail::vector::calls::types::SendMessage;

use avail::{
	runtime_types::bounded_collections::bounded_vec::BoundedVec,
	vector::calls::types::{
		execute::{AccountProof, AddrMessage, StorageProof},
		send_message::Message,
	},
};

#[derive(Clone)]
pub struct Vector {
	online_client: AOnlineClient,
	rpc_client: RpcClient,
}

impl Vector {
	pub fn new(online_client: AOnlineClient, rpc_client: RpcClient) -> Self {
		Self {
			online_client,
			rpc_client,
		}
	}

	pub fn execute(
		&self,
		slot: u64,
		addr_message: AddrMessage,
		account_proof: AccountProof,
		storage_proof: StorageProof,
	) -> Transaction<ExecuteCall> {
		let payload =
			avail::tx()
				.vector()
				.execute(slot, addr_message, account_proof, storage_proof);
		Transaction::new(self.online_client.clone(), self.rpc_client.clone(), payload)
	}

	pub fn fulfill_call(
		&self,
		function_id: H256,
		input: Vec<u8>,
		output: Vec<u8>,
		proof: Vec<u8>,
		slot: u64,
	) -> Transaction<FulfillCallCall> {
		let input = BoundedVec { 0: input };
		let output = BoundedVec { 0: output };
		let proof = BoundedVec { 0: proof };

		let payload = avail::tx()
			.vector()
			.fulfill_call(function_id, input, output, proof, slot);
		Transaction::new(self.online_client.clone(), self.rpc_client.clone(), payload)
	}

	pub fn send_message(
		&self,
		message: Message,
		to: H256,
		domain: u32,
	) -> Transaction<SendMessageCall> {
		let payload = avail::tx().vector().send_message(message, to, domain);
		Transaction::new(self.online_client.clone(), self.rpc_client.clone(), payload)
	}
}
