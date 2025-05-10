use primitive_types::H256;
use subxt_signer::sr25519::Keypair;

use super::Client;
use crate::{error::RpcError, prelude::Options, primitives, SubmittedTransaction};

impl primitives::TransactionCall {
	pub async fn sign_and_submit(
		&self,
		client: &Client,
		signer: &Keypair,
		options: Options,
	) -> Result<SubmittedTransaction, RpcError> {
		client.sign_and_submit_call(signer, self, options).await
	}
}
impl<'a> primitives::TransactionPayload<'a> {
	pub async fn sign_and_submit(&self, client: &Client, signer: &Keypair) -> Result<H256, RpcError> {
		client.sign_and_submit(signer, self.clone()).await
	}
}
impl<'a> primitives::Transaction<'a> {
	pub async fn sign_and_submit(&self, client: &Client) -> Result<H256, RpcError> {
		let encoded = self.encode();
		client.rpc_author_submit_extrinsic(&encoded).await
	}
}
