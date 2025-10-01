//! Subscriptions that stream GRANDPA justifications in both binary and JSON-encoded forms.

use crate::{Client, GrandpaJustification, RpcError, Sub};
use std::time::Duration;

/// Streams decoded GRANDPA justifications while handling retries and cursor management.
#[derive(Clone)]
pub struct GrandpaJustificationSub {
	sub: Sub,
}

impl GrandpaJustificationSub {
	/// Creates a subscription that yields decoded GRANDPA justifications.
	///
	/// The client is cloned; no network calls are made until [`GrandpaJustificationSub::next`] is
	/// awaited.
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	/// Fetches the next available justification; rewinds on RPC failure and skips blanks.
	///
	/// # Returns
	/// - `Ok(GrandpaJustification)` when a justification is retrieved for the current cursor.
	/// - `Err(RpcError)` when the RPC request fails. The internal block height rewinds so the same
	///   block is retried on the next call.
	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let info = self.sub.next().await?;
			let just = match self.fetch_justification(info.height).await {
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

	/// Jump the cursor to a specific starting height. The next call to [`GrandpaJustificationSub::next`]
	/// examines this height first.
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Change how often the subscription polls for new justifications when tailing the head.
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Controls retry behaviour for subsequent RPC calls (`Some(true)` = force, `Some(false)` = disable,
	/// `None` = inherit client default).
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}

	/// Returns true when this stream retries failed justification RPC calls.
	pub fn should_retry_on_error(&self) -> bool {
		self.sub.should_retry_on_error()
	}

	async fn fetch_justification(&self, height: u32) -> Result<Option<GrandpaJustification>, RpcError> {
		self.sub
			.client_ref()
			.chain()
			.retry_on(Some(self.should_retry_on_error()), None)
			.grandpa_block_justification(height)
			.await
	}
}

/// Streams GRANDPA justifications encoded as JSON structures.
#[derive(Clone)]
pub struct GrandpaJustificationJsonSub {
	sub: Sub,
}

impl GrandpaJustificationJsonSub {
	/// Creates a subscription that yields JSON GRANDPA justifications. Network calls are deferred until
	/// [`GrandpaJustificationJsonSub::next`] is awaited.
	pub fn new(client: Client) -> Self {
		Self { sub: Sub::new(client) }
	}

	/// Fetches the next available justification; rewinds on RPC failure and skips blanks.
	///
	/// Return semantics mirror [`GrandpaJustificationSub::next`].
	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let info = self.sub.next().await?;
			let just = match self.fetch_justification(info.height).await {
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

	/// Jump the cursor to a specific starting height.
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Change how often the subscription polls for new justifications when following the head.
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Controls retry behaviour (`Some(true)` = force retries, `Some(false)` = disable, `None` =
	/// inherit client default).
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}

	/// Returns true when this stream retries failed justification RPC calls.
	pub fn should_retry_on_error(&self) -> bool {
		self.sub.should_retry_on_error()
	}

	async fn fetch_justification(&self, height: u32) -> Result<Option<GrandpaJustification>, RpcError> {
		self.sub
			.client_ref()
			.chain()
			.retry_on(Some(self.should_retry_on_error()), None)
			.grandpa_block_justification_json(height)
			.await
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::{clients::mock_client::MockClient, error::Error, prelude::*, subxt_rpcs::RpcClient};

	#[tokio::test]
	async fn grandpa_justification_sub_test() -> Result<(), Error> {
		_ = Client::init_tracing(false);
		let (rpc_client, mut commander) = MockClient::new(MAINNET_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

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
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

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
