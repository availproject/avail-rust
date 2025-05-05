use crate::{avail, client::Client, SubmittableTransaction};
use primitive_types::H256;

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
	pub client: Client,
}

impl Vector {
	pub fn execute(
		&self,
		slot: u64,
		addr_message: AddrMessage,
		account_proof: AccountProof,
		storage_proof: StorageProof,
	) -> SubmittableTransaction<ExecuteCall> {
		let payload = avail::tx()
			.vector()
			.execute(slot, addr_message, account_proof, storage_proof);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	pub fn fulfill_call(
		&self,
		function_id: H256,
		input: Vec<u8>,
		output: Vec<u8>,
		proof: Vec<u8>,
		slot: u64,
	) -> SubmittableTransaction<FulfillCallCall> {
		let input = BoundedVec(input);
		let output = BoundedVec(output);
		let proof = BoundedVec(proof);

		let payload = avail::tx()
			.vector()
			.fulfill_call(function_id, input, output, proof, slot);
		SubmittableTransaction::new(self.client.clone(), payload)
	}

	pub fn send_message(&self, message: Message, to: H256, domain: u32) -> SubmittableTransaction<SendMessageCall> {
		let payload = avail::tx().vector().send_message(message, to, domain);
		SubmittableTransaction::new(self.client.clone(), payload)
	}
}
