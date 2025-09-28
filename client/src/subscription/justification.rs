use crate::{Client, GrandpaJustification, RpcError, Sub};
use std::time::Duration;

#[derive(Clone)]
pub struct GrandpaJustificationSub {
	sub: Sub,
}

impl GrandpaJustificationSub {
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let info = self.sub.next().await?;
			let retry = self.sub.should_retry_on_error();
			let just = match self.fetch_justification(info.height, retry).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			let Some(just) = just else {
				continue;
			};

			return Ok(just);
		}
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}

	async fn fetch_justification(&self, height: u32, retry: bool) -> Result<Option<GrandpaJustification>, RpcError> {
		self.sub
			.client_ref()
			.rpc()
			.retry_on(Some(retry), None)
			.grandpa_block_justification(height)
			.await
	}
}

#[derive(Clone)]
pub struct GrandpaJustificationJsonSub {
	sub: Sub,
}

impl GrandpaJustificationJsonSub {
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let info = self.sub.next().await?;
			let retry = self.sub.should_retry_on_error();
			let just = match self.fetch_justification(info.height, retry).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			let Some(just) = just else {
				continue;
			};

			return Ok(just);
		}
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}

	async fn fetch_justification(&self, height: u32, retry: bool) -> Result<Option<GrandpaJustification>, RpcError> {
		self.sub
			.client_ref()
			.rpc()
			.retry_on(Some(retry), None)
			.grandpa_block_justification_json(height)
			.await
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use super::*;

	use crate::{clients::mock_client::MockClient, error::Error, prelude::*, subxt_rpcs::RpcClient};

	#[tokio::test]
	async fn grandpa_justification_sub_test() -> Result<(), Error> {
		_ = Client::init_tracing(false);
		let (rpc_client, mut commander) = MockClient::new(MAINNET_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client), Arc::new(StandardAsyncOp)).await?;

		// Historical block
		let mut sub = GrandpaJustificationSub::new(client.clone());

		sub.set_block_height(1900031);
		let n = sub.next().await?;
		assert_eq!(n.commit.target_number, 1900032);

		sub.set_block_height(1900122);
		let n = sub.next().await?;
		assert_eq!(n.commit.target_number, 1900122);

		// Testing recovery
		sub.set_block_height(1);
		assert_eq!(sub.sub.as_finalized().next_block_height, 1);

		// 1 is Ok(Some)
		// 2 is Ok(None)
		// 3 is Ok(Some)
		// 4 is Err
		// 4 is Ok(Some)
		commander.justification_ok(Some(GrandpaJustification::default())); // 1
		commander.justification_ok(None); // 2
		commander.justification_ok(Some(GrandpaJustification::default())); // 3
		commander.justification_err(None); // 4
		commander.justification_ok(Some(GrandpaJustification::default())); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(Some(false));
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}

	#[tokio::test]
	async fn grandpa_justification_json_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(MAINNET_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client), Arc::new(StandardAsyncOp)).await?;

		// Historical block
		let mut sub = GrandpaJustificationJsonSub::new(client.clone());

		sub.set_block_height(1900031);
		let n = sub.next().await?;
		assert_eq!(n.commit.target_number, 1900032);

		sub.set_block_height(1900122);
		let n = sub.next().await?;
		assert_eq!(n.commit.target_number, 1900122);

		// Testing recovery
		sub.set_block_height(1);
		assert_eq!(sub.sub.as_finalized().next_block_height, 1);

		// 1 is Ok(Some)
		// 2 is Ok(None)
		// 3 is Ok(Some)
		// 4 is Err
		// 4 is Ok(Some)
		commander.justification_json_ok(Some(GrandpaJustification::default())); // 1
		commander.justification_json_ok(None); // 2
		commander.justification_json_ok(Some(GrandpaJustification::default())); // 3
		commander.justification_json_err(None); // 4
		commander.justification_json_ok(Some(GrandpaJustification::default())); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(Some(false));
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}
}
