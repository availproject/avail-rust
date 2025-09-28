use crate::{
	BlockRef, Client, Sub,
	block::{
		BlockExtOptionsExpanded, BlockExtOptionsSimple, BlockExtrinsic, BlockRawExtrinsic, BlockTransaction,
		BlockWithExt, BlockWithRawExt, BlockWithTx,
	},
};
use avail_rust_core::HasHeader;
use codec::Decode;
use std::{marker::PhantomData, time::Duration};

/// Subscription that mirrors [`Sub`] but yields decoded transactions via [`BlockWithTx`].
///
/// The iterator skips blocks without matching transactions so callers only handle blocks that
/// produced data when applying the configured [`BlockExtOptionsSimple`].
#[derive(Clone)]
pub struct TransactionSub<T: HasHeader + Decode> {
	sub: Sub,
	opts: BlockExtOptionsSimple,
	_phantom: PhantomData<T>,
}

impl<T: HasHeader + Decode> TransactionSub<T> {
	/// Creates a new [`TransactionSub`] subscription.
	pub fn new(client: Client, opts: BlockExtOptionsSimple) -> Self {
		Self { sub: Sub::new(client), opts, _phantom: Default::default() }
	}

	/// Returns the next set of block transactions and the corresponding [`BlockRef`].
	///
	/// Empty responses are skipped automatically. When fetching fails the internal block height is
	/// rewound so the same block can be retried on the following call.
	pub async fn next(&mut self) -> Result<(Vec<BlockTransaction<T>>, BlockRef), crate::Error> {
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
	pub fn set_opts(&mut self, value: BlockExtOptionsSimple) {
		self.opts = value;
	}

	/// Delegates to [`Sub::use_best_block`].
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Delegates to [`Sub::set_block_height`].
	pub fn set_block_height(&mut self, value: u32) {
		self.sub.set_block_height(value);
	}

	/// Delegates to [`Sub::set_pool_rate`].
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Delegates to [`Sub::set_retry_on_error`].
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

/// Subscription that mirrors [`Sub`] but yields decoded extrinsics via [`BlockWithExt`].
///
/// Blocks without matching extrinsics are skipped so every returned item contains data along with
/// its [`BlockRef`].
#[derive(Clone)]
pub struct ExtrinsicSub<T: HasHeader + Decode> {
	sub: Sub,
	opts: BlockExtOptionsSimple,
	_phantom: PhantomData<T>,
}

impl<T: HasHeader + Decode> ExtrinsicSub<T> {
	/// Creates a new [`ExtrinsicSub`] subscription.
	pub fn new(client: Client, opts: BlockExtOptionsSimple) -> Self {
		Self { sub: Sub::new(client), opts, _phantom: Default::default() }
	}

	/// Returns the next collection of extrinsics and its [`BlockRef`].
	///
	/// Empty responses trigger another iteration. Failed RPC calls reset the internal block height so
	/// the same block can be retried.
	pub async fn next(&mut self) -> Result<(Vec<BlockExtrinsic<T>>, BlockRef), crate::Error> {
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

	/// Delegates to [`Sub::use_best_block`].
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Delegates to [`Sub::set_block_height`].
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Delegates to [`Sub::set_pool_rate`].
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Delegates to [`Sub::set_retry_on_error`].
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

/// Subscription that mirrors [`Sub`] but provides raw extrinsic payloads via [`BlockWithRawExt`].
///
/// Useful when you want the raw data from the extrinsic rpc.
/// Blocks without matching extrinsics are skipped so every returned item contains data along with
/// its [`BlockRef`].
#[derive(Clone)]
pub struct RawExtrinsicSub {
	sub: Sub,
	opts: BlockExtOptionsExpanded,
}

impl RawExtrinsicSub {
	/// Creates a new [`RawExtrinsicSub`] subscription.
	pub fn new(client: Client, opts: BlockExtOptionsExpanded) -> Self {
		Self { sub: Sub::new(client), opts }
	}

	/// Returns the next batch of raw extrinsics and its [`BlockRef`].
	///
	/// Empty results are skipped. Failed RPC calls reset the internal block height so the same block
	/// can be retried.
	pub async fn next(&mut self) -> Result<(Vec<BlockRawExtrinsic>, BlockRef), crate::Error> {
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

	/// Delegates to [`Sub::use_best_block`].
	pub fn use_best_block(&mut self, value: bool) {
		self.sub.use_best_block(value);
	}

	/// Delegates to [`Sub::set_block_height`].
	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	/// Delegates to [`Sub::set_pool_rate`].
	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	/// Delegates to [`Sub::set_retry_on_error`].
	pub fn set_retry_on_error(&mut self, value: Option<bool>) {
		self.sub.set_retry_on_error(value);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		block::BlockExtOptionsExpanded, clients::mock_client::MockClient, error::Error, prelude::*,
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
