//! Subscription adapters that surface extrinsics and transactions in decoded or raw form.

use crate::{
	BlockInfo, Client, Sub,
	block_api::{
		BlockExtOptionsExpanded, BlockExtOptionsSimple, BlockExtrinsic, BlockRawExtrinsic, BlockTransaction,
		BlockWithExt, BlockWithRawExt, BlockWithTx,
	},
};
use avail_rust_core::HasHeader;
use codec::Decode;
use std::{marker::PhantomData, time::Duration};

/// Subscription that mirrors [`Sub`] but yields decoded transactions via [`BlockWithTx`].
///
/// Blocks that do not produce transactions matching the provided options are skipped automatically,
/// ensuring callers only handle meaningful batches.
#[derive(Clone)]
pub struct TransactionSub<T: HasHeader + Decode> {
	sub: Sub,
	opts: BlockExtOptionsSimple,
	_phantom: PhantomData<T>,
}

impl<T: HasHeader + Decode> TransactionSub<T> {
	/// Creates a new [`TransactionSub`] subscription.
	///
	/// The client is cloned and no network calls are performed until [`TransactionSub::next`] is
	/// awaited. `opts` controls which transactions are retrieved from each block.
	pub fn new(client: Client, opts: BlockExtOptionsSimple) -> Self {
		Self { sub: Sub::new(client), opts, _phantom: Default::default() }
	}

	/// Returns the next set of block transactions and the corresponding [`BlockInfo`].
	///
	/// # Returns
	/// - `Ok((Vec<BlockTransaction<T>>, BlockInfo))` when a block with matching transactions is found;
	///   the vector is guaranteed to be non-empty.
	/// - `Err(crate::Error)` when fetching fails. The cursor rewinds to the same block so a subsequent
	///   call can retry.
	///
	/// On success the subscription advances to the next block height.
	pub async fn next(&mut self) -> Result<(Vec<BlockTransaction<T>>, BlockInfo), crate::Error> {
		loop {
			let info = self.sub.next().await?;
			let mut block = BlockWithTx::new(self.sub.client_ref().clone(), info.hash);
			block.set_retry_on_error(Some(self.sub.should_retry_on_error()));

			let txs = match block.all::<T>(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			if txs.is_empty() {
				continue;
			}

			return Ok((txs, info));
		}
	}

	/// Replaces the transaction query options used on subsequent calls to [`TransactionSub::next`].
	/// The change takes effect immediately.
	pub fn set_opts(&mut self, value: BlockExtOptionsSimple) {
		self.opts = value;
	}

	/// Follow best blocks instead of finalized ones for future iterations that have not yet been
	/// executed.
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Jump the cursor to a specific starting height. The next call to [`TransactionSub::next`] begins
	/// evaluating from `value`.
	pub fn set_block_height(&mut self, value: u32) {
		self.sub.set_block_height(value);
	}

	/// Change how often new blocks are polled when tailing the chain. Historical replays are not
	/// affected by this interval.
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Controls retry behaviour for future RPC calls issued by the subscription.
	///
	/// - `Some(true)`: force retries regardless of the client's global setting.
	/// - `Some(false)`: disable retries entirely.
	/// - `None`: defer to the client's configuration.
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}

	/// Returns true when this subscription will retry failed RPC calls.
	pub fn should_retry_on_error(&self) -> bool {
		self.sub.should_retry_on_error()
	}
}

/// Subscription that mirrors [`Sub`] but yields decoded extrinsics via [`BlockWithExt`].
///
/// Blocks without matching extrinsics are skipped so every returned item contains data along with
/// its [`BlockInfo`].
#[derive(Clone)]
pub struct ExtrinsicSub<T: HasHeader + Decode> {
	sub: Sub,
	opts: BlockExtOptionsSimple,
	_phantom: PhantomData<T>,
}

impl<T: HasHeader + Decode> ExtrinsicSub<T> {
	/// Creates a new [`ExtrinsicSub`] subscription.
	///
	/// No network calls are issued until the first [`ExtrinsicSub::next`] invocation.
	pub fn new(client: Client, opts: BlockExtOptionsSimple) -> Self {
		Self { sub: Sub::new(client), opts, _phantom: Default::default() }
	}

	/// Returns the next collection of extrinsics and its [`BlockInfo`].
	///
	/// # Returns
	/// - `Ok((Vec<BlockExtrinsic<T>>, BlockInfo))` when a block contains extrinsics matching the
	///   configured options.
	/// - `Err(crate::Error)` when the RPC query fails. The internal cursor rewinds so a retry will re-
	///   attempt the same block.
	pub async fn next(&mut self) -> Result<(Vec<BlockExtrinsic<T>>, BlockInfo), crate::Error> {
		loop {
			let info = self.sub.next().await?;
			let mut block = BlockWithExt::new(self.sub.client_ref().clone(), info.hash);
			block.set_retry_on_error(Some(self.sub.should_retry_on_error()));

			let txs = match block.all::<T>(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			if txs.is_empty() {
				continue;
			}

			return Ok((txs, info));
		}
	}

	/// Follow best blocks instead of finalized ones for future iterations.
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Jump the cursor to a specific starting height.
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Change how often the subscription polls for new blocks when tailing the chain.
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Controls retry behaviour for future RPC calls (`Some(true)` = force, `Some(false)` = disable).
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}

	/// Returns true when this subscription will retry failed RPC calls.
	pub fn should_retry_on_error(&self) -> bool {
		self.sub.should_retry_on_error()
	}
}

/// Subscription that mirrors [`Sub`] but provides raw extrinsic payloads via [`BlockWithRawExt`].
///
/// Useful when you want the raw data from the extrinsic rpc.
/// Blocks without matching extrinsics are skipped so every returned item contains data along with
/// its [`BlockInfo`].
#[derive(Clone)]
pub struct RawExtrinsicSub {
	sub: Sub,
	opts: BlockExtOptionsExpanded,
}

impl RawExtrinsicSub {
	/// Creates a new [`RawExtrinsicSub`] subscription.
	///
	/// The supplied options control filters and encoding preferences. No network calls are made until
	/// [`RawExtrinsicSub::next`] is awaited.
	pub fn new(client: Client, opts: BlockExtOptionsExpanded) -> Self {
		Self { sub: Sub::new(client), opts }
	}

	/// Returns the next batch of raw extrinsics and its [`BlockInfo`].
	///
	/// # Returns
	/// - `Ok((Vec<BlockRawExtrinsic>, BlockInfo))` with at least one element when a block matches the
	///   filter.
	/// - `Err(crate::Error)` when fetching fails; the cursor rewinds to retry the same block on the next
	///   call.
	pub async fn next(&mut self) -> Result<(Vec<BlockRawExtrinsic>, BlockInfo), crate::Error> {
		loop {
			let info = self.sub.next().await?;
			let mut block = BlockWithRawExt::new(self.sub.client_ref().clone(), info.hash);
			block.set_retry_on_error(Some(self.sub.should_retry_on_error()));

			let txs = match block.all(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch transactions
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			if txs.is_empty() {
				continue;
			}

			return Ok((txs, info));
		}
	}

	/// Follow best blocks instead of finalized ones when scanning forward.
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Jump the cursor to a specific starting height before the next fetch.
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Change how often the subscription polls for new blocks when following the head.
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Choose whether this subscription should retry after RPC failures (`Some(true)` = force,
	/// `Some(false)` = disable, `None` = inherit client default).
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}

	/// Returns true when this subscription will retry failed RPC calls.
	pub fn should_retry_on_error(&self) -> bool {
		self.sub.should_retry_on_error()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		block_api::BlockExtOptionsExpanded, clients::mock_client::MockClient, error::Error, prelude::*,
		subxt_rpcs::RpcClient,
	};
	use avail_rust_core::{
		avail::data_availability::tx::SubmitData, rpc::system::fetch_extrinsics::ExtrinsicInformation,
	};

	#[tokio::test]
	async fn transaction_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical blocks
		let mut sub = TransactionSub::<SubmitData>::new(client.clone(), Default::default());

		sub.set_block_height(2326671);
		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326672);
		assert_eq!(list.len(), 1);

		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326674);
		assert_eq!(list.len(), 1);

		// Testing recovery
		sub.set_block_height(1);
		assert_eq!(sub.sub.as_finalized().next_block_height, 1);

		// 1 is Ok(Some)
		// 2 is Ok(None)
		// 3 is Ok(Some)
		// 4 is Err
		// 4 is Ok(Some)
		let mut data = ExtrinsicInformation::default();
		let tx = client.tx().data_availability().submit_data("1234");
		data.encoded = Some(const_hex::encode(tx.sign(&alice(), Options::new(2)).await?.encode()));

		commander.extrinsics_ok(vec![data.clone()]); // 1
		commander.extrinsics_ok(vec![]); // 2
		commander.extrinsics_ok(vec![data.clone()]); // 3
		commander.extrinsics_err(None); // 4
		commander.extrinsics_ok(vec![data.clone()]); // 4

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
	async fn extrinsic_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical blocks
		let mut sub = ExtrinsicSub::<SubmitData>::new(client.clone(), Default::default());

		sub.set_block_height(2326671);
		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326672);
		assert_eq!(list.len(), 1);

		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326674);
		assert_eq!(list.len(), 1);

		// Testing recovery
		sub.set_block_height(1);
		assert_eq!(sub.sub.as_finalized().next_block_height, 1);

		// 1 is Ok(Some)
		// 2 is Ok(None)
		// 3 is Ok(Some)
		// 4 is Err
		// 4 is Ok(Some)
		let mut data = ExtrinsicInformation::default();
		let tx = client.tx().data_availability().submit_data("1234");
		data.encoded = Some(const_hex::encode(tx.sign(&alice(), Options::new(2)).await?.encode()));

		commander.extrinsics_ok(vec![data.clone()]); // 1
		commander.extrinsics_ok(vec![]); // 2
		commander.extrinsics_ok(vec![data.clone()]); // 3
		commander.extrinsics_err(None); // 4
		commander.extrinsics_ok(vec![data.clone()]); // 4

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
	async fn raw_extrinsic_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::from_rpc_client(RpcClient::new(rpc_client)).await?;

		// Historical blocks
		let opts = BlockExtOptionsExpanded { filter: Some((29u8, 1u8).into()), ..Default::default() };
		let mut sub = RawExtrinsicSub::new(client.clone(), opts);

		sub.set_block_height(2326671);
		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326672);
		assert_eq!(list.len(), 1);

		let (list, info) = sub.next().await?;
		assert_eq!(info.height, 2326674);
		assert_eq!(list.len(), 1);

		// Testing recovery
		sub.set_block_height(1);
		assert_eq!(sub.sub.as_finalized().next_block_height, 1);

		// 1 is Ok(Some)
		// 2 is Ok(None)
		// 3 is Ok(Some)
		// 4 is Err
		// 4 is Ok(Some)
		let mut data = ExtrinsicInformation::default();
		let tx = client.tx().data_availability().submit_data("1234");
		data.encoded = Some(const_hex::encode(tx.sign(&alice(), Options::new(2)).await?.encode()));

		commander.extrinsics_ok(vec![data.clone()]); // 1
		commander.extrinsics_ok(vec![]); // 2
		commander.extrinsics_ok(vec![data.clone()]); // 3
		commander.extrinsics_err(None); // 4
		commander.extrinsics_ok(vec![data.clone()]); // 4

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
