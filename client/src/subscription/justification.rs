//! Subscriptions that stream GRANDPA justifications in both binary and JSON-encoded forms.

use avail_rust_core::H256;

use crate::{Client, GrandpaJustification, RpcError, Sub};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct GrandpaJustificationSubValue {
	pub value: Option<GrandpaJustification>,
	pub block_height: u32,
	pub block_hash: H256,
}

/// Streams decoded GRANDPA justifications while handling retries and cursor management.
///
/// Internally this wrapper keeps track of the next block height to inspect and mirrors the retry
/// settings of the underlying [`Client`], making it suitable for long-lived background tasks.
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

	/// Advances to the next block and returns its justification (if present) alongside metadata.
	///
	/// The returned tuple contains an optional [`GrandpaJustification`] for the finalized block plus
	/// the block hash and height describing that block. When `None` is returned it means the block finalized
	/// without an available justification, which is expected for some historical heights. Errors
	/// propagate from the underlying RPC, respecting the retry policy set via
	/// [`GrandpaJustificationSub::set_retry_on_error`]. On failure, the internal cursor is rewound so
	/// the same height is retried on the next call.
	pub async fn next(&mut self) -> Result<GrandpaJustificationSubValue, RpcError> {
		let info = self.sub.next().await?;
		let justification = match self.fetch_justification(info.height).await {
			Ok(x) => x,
			Err(err) => {
				// Revet block height if we fail to fetch transactions
				self.sub.set_block_height(info.height);
				return Err(err);
			},
		};

		return Ok(GrandpaJustificationSubValue {
			value: justification,
			block_hash: info.hash,
			block_height: info.height,
		});
	}

	pub async fn prev(&mut self) -> Result<GrandpaJustificationSubValue, RpcError> {
		let info = self.sub.prev().await?;
		let justification = match self.fetch_justification(info.height).await {
			Ok(x) => x,
			Err(err) => {
				// Revet block height if we fail to fetch transactions
				self.sub.set_block_height(info.height);
				return Err(err);
			},
		};

		return Ok(GrandpaJustificationSubValue {
			value: justification,
			block_hash: info.hash,
			block_height: info.height,
		});
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
		let retry = Some(self.should_retry_on_error());
		let chain = self.sub.client_ref().chain().retry_on(retry, None);
		chain.grandpa_block_justification(height).await
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
		assert_eq!(n.block_height, 1900031);
		assert!(n.value.is_none());

		sub.set_block_height(1900122);
		let n = sub.next().await?;
		assert_eq!(n.block_height, 1900122);
		assert_eq!(n.value.unwrap().commit.target_number, 1900122);

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

		let v = sub.next().await?;
		assert!(v.value.is_some());
		assert_eq!(v.block_height, 1);

		let v = sub.next().await?;
		assert!(v.value.is_none());
		assert_eq!(v.block_height, 2);

		let v = sub.next().await?;
		assert!(v.value.is_some());
		assert_eq!(v.block_height, 3);

		sub.set_retry_on_error(Some(false));
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let v = sub.next().await?;
		assert!(v.value.is_some());
		assert_eq!(v.block_height, 4);

		Ok(())
	}
}
