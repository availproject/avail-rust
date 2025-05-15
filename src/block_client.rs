use crate::{client::Client, error::RpcError, primitives, H256};

#[derive(Clone)]
pub struct BlockClient {
	client: Client,
}

impl BlockClient {
	pub fn new(client: Client) -> Self {
		Self { client }
	}

	pub async fn block(&self, at: H256) -> Result<Option<primitives::rpc::Block>, RpcError> {
		self.client.block(at).await.map(|b| b.map(|x| x.block))
	}

	pub async fn best_block(&self) -> Result<primitives::rpc::Block, RpcError> {
		self.client.best_block().await.map(|b| b.block)
	}

	pub async fn finalized_block(&self) -> Result<primitives::rpc::Block, RpcError> {
		self.client
			.finalized_block()
			.await
			.map(|b: primitives::rpc::SignedBlock| b.block)
	}
}
