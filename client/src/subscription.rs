use crate::{
	AvailHeader, Client,
	block::{
		Block, BlockEvents, BlockEventsOptions, BlockExtOptionsExpanded, BlockExtOptionsSimple, BlockExtrinsic,
		BlockRawExtrinsic, BlockTransaction, BlockWithExt, BlockWithRawExt, BlockWithTx,
	},
	platform::sleep,
};
use avail_rust_core::{
	BlockRef, H256, HasHeader, RpcError,
	grandpa::GrandpaJustification,
	rpc::{BlockPhaseEvent, BlockWithJustifications},
};
use codec::Decode;
use std::{marker::PhantomData, time::Duration};

#[derive(Clone)]
pub struct UnInitSub {
	pub(crate) use_best_block: bool,
	pub(crate) block_height: Option<u32>,
	pub(crate) poll_rate: Duration,
	pub(crate) retry_on_error: bool,
}

impl UnInitSub {
	pub fn new() -> Self {
		Self::default()
	}

	pub async fn build(&self, client: &Client) -> Result<Sub, RpcError> {
		let block_height = match self.block_height {
			Some(x) => x,
			None => match self.use_best_block {
				true => client.best().block_height().await?,
				false => client.finalized().block_height().await?,
			},
		};

		let sub = match self.use_best_block {
			true => Sub::BestBlock(BestBlockSub {
				poll_rate: self.poll_rate,
				current_block_height: block_height,
				block_processed: Vec::new(),
				retry_on_error: self.retry_on_error,
				latest_finalized_height: None,
			}),
			false => Sub::FinalizedBlock(FinalizedBlockSub {
				poll_rate: self.poll_rate,
				next_block_height: block_height,
				retry_on_error: self.retry_on_error,
				latest_finalized_height: None,
			}),
		};
		Ok(sub)
	}
}

impl Default for UnInitSub {
	fn default() -> Self {
		Self {
			use_best_block: false,
			block_height: Default::default(),
			poll_rate: Duration::from_secs(3),
			retry_on_error: true,
		}
	}
}

#[derive(Clone)]
pub struct FinalizedBlockSub {
	poll_rate: Duration,
	next_block_height: u32,
	retry_on_error: bool,
	latest_finalized_height: Option<u32>,
}

impl FinalizedBlockSub {
	pub async fn run(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		let latest_finalized_height = self.fetch_latest_finalized_height(client).await?;

		let result = if latest_finalized_height >= self.next_block_height {
			self.run_historical(client).await?
		} else {
			self.run_head(client).await?
		};

		self.next_block_height = result.height + 1;
		Ok(result)
	}

	async fn fetch_latest_finalized_height(&mut self, client: &Client) -> Result<u32, RpcError> {
		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		let latest_finalized_height = client.finalized().block_height().await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	async fn run_historical(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		let height = self.next_block_height;
		let hash = client
			.rpc()
			.retry_on(Some(self.retry_on_error), None)
			.block_hash(Some(height))
			.await?;
		let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

		Ok(BlockRef { hash, height })
	}

	async fn run_head(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		loop {
			let head = client.finalized().block_info().await?;

			let is_past_block = self.next_block_height > head.height;
			if is_past_block {
				sleep(self.poll_rate).await;
				continue;
			}

			if self.next_block_height == head.height {
				return Ok(head);
			}

			let height = self.next_block_height;
			let hash = client
				.rpc()
				.retry_on(Some(self.retry_on_error), Some(true))
				.block_hash(Some(height))
				.await?;
			let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

			return Ok(BlockRef { hash, height });
		}
	}
}

#[derive(Clone)]
pub struct BestBlockSub {
	poll_rate: Duration,
	current_block_height: u32,
	block_processed: Vec<H256>,
	retry_on_error: bool,
	latest_finalized_height: Option<u32>,
}

impl BestBlockSub {
	pub async fn run(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		let latest_finalized_height = self.fetch_latest_finalized_height(client).await?;

		// Dealing with historical blocks
		if latest_finalized_height >= self.current_block_height {
			let info = self.run_historical(client).await?;
			self.current_block_height = info.height + 1;
			return Ok(info);
		}

		let info = self.run_head(client).await?;
		if info.height == self.current_block_height {
			self.block_processed.push(info.hash);
		} else {
			self.block_processed = vec![info.hash];
			self.current_block_height = info.height;
		}

		Ok(info)
	}

	async fn fetch_latest_finalized_height(&mut self, client: &Client) -> Result<u32, RpcError> {
		if let Some(height) = self.latest_finalized_height.as_ref() {
			return Ok(*height);
		}

		let latest_finalized_height = client.finalized().block_height().await?;
		self.latest_finalized_height = Some(latest_finalized_height);
		Ok(latest_finalized_height)
	}

	async fn run_historical(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		let height = self.current_block_height;
		let hash = client
			.rpc()
			.retry_on(Some(self.retry_on_error), None)
			.block_hash(Some(height))
			.await?;
		let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

		Ok(BlockRef { hash, height })
	}

	async fn run_head(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		loop {
			let head = client.best().block_info().await?;

			let is_past_block = self.current_block_height > head.height;
			let block_already_processed = self.block_processed.contains(&head.hash);
			if is_past_block || block_already_processed {
				sleep(self.poll_rate).await;
				continue;
			}

			let is_current_block = self.current_block_height == head.height;
			let is_next_block = self.current_block_height + 1 == head.height;
			if is_current_block || is_next_block {
				return Ok(head);
			}

			let no_block_processed_yet = self.block_processed.is_empty();
			if no_block_processed_yet {
				let hash = client
					.rpc()
					.retry_on(Some(true), Some(true))
					.block_hash(Some(self.current_block_height))
					.await?;
				let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

				return Ok(BlockRef { hash, height: self.current_block_height });
			}

			let height = self.current_block_height + 1;
			let hash = client
				.rpc()
				.retry_on(Some(true), Some(true))
				.block_hash(Some(height))
				.await?;
			let hash = hash.ok_or(RpcError::ExpectedData("Expected to fetch block hash".into()))?;

			return Ok(BlockRef { hash, height });
		}
	}
}

#[derive(Clone)]
pub enum Sub {
	UnInit(UnInitSub),
	BestBlock(BestBlockSub),
	FinalizedBlock(FinalizedBlockSub),
}

impl Sub {
	pub fn new() -> Self {
		Self::default()
	}

	pub async fn next(&mut self, client: &Client) -> Result<BlockRef, RpcError> {
		if let Self::UnInit(u) = self {
			let concrete = u.build(client).await?;
			*self = concrete;
		};

		match self {
			Self::BestBlock(s) => s.run(client).await,
			Self::FinalizedBlock(s) => s.run(client).await,
			_ => unreachable!("We cannot be here."),
		}
	}

	pub fn retry_on_error(&self) -> bool {
		match self {
			Self::UnInit(u) => u.retry_on_error,
			Self::BestBlock(s) => s.retry_on_error,
			Self::FinalizedBlock(s) => s.retry_on_error,
		}
	}

	pub fn set_follow(&mut self, best_block: bool) {
		if let Self::UnInit(u) = self {
			u.use_best_block = best_block;
		}
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		match self {
			Self::UnInit(u) => u.block_height = Some(block_height),
			Self::BestBlock(x) => {
				x.block_processed.clear();
				x.current_block_height = block_height;
			},
			Self::FinalizedBlock(x) => x.next_block_height = block_height,
		}
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		match self {
			Self::UnInit(u) => u.poll_rate = value,
			Self::BestBlock(x) => x.poll_rate = value,
			Self::FinalizedBlock(x) => x.poll_rate = value,
		}
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		match self {
			Self::UnInit(u) => u.retry_on_error = value,
			Self::BestBlock(x) => x.retry_on_error = value,
			Self::FinalizedBlock(x) => x.retry_on_error = value,
		}
	}

	#[cfg(test)]
	fn as_finalized(&self) -> &FinalizedBlockSub {
		if let Self::FinalizedBlock(f) = self {
			return f;
		}
		panic!("Not Finalized Sub");
	}
}

impl Default for Sub {
	fn default() -> Self {
		Self::UnInit(UnInitSub::default())
	}
}

#[derive(Clone)]
pub struct BlockWithJustSub {
	client: Client,
	sub: Sub,
}

impl BlockWithJustSub {
	pub fn new(client: Client) -> Self {
		Self { client, sub: Sub::new() }
	}

	pub async fn next(&mut self) -> Result<Option<BlockWithJustifications>, RpcError> {
		let info = self.sub.next(&self.client).await?;
		let block = match self
			.client
			.rpc()
			.retry_on(Some(self.sub.retry_on_error()), Some(true))
			.block_with_justification(Some(info.hash))
			.await
		{
			Ok(x) => x,
			Err(err) => {
				// Revet block height if we fail to fetch block
				self.sub.set_block_height(info.height);
				return Err(err);
			},
		};
		Ok(block)
	}

	pub fn set_follow(&mut self, best_block: bool) {
		self.sub.set_follow(best_block);
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct BlockSub {
	client: Client,
	sub: Sub,
}

impl BlockSub {
	pub fn new(client: Client) -> Self {
		Self { client, sub: Sub::new() }
	}

	pub async fn next(&mut self) -> Result<(Block, BlockRef), RpcError> {
		let info = self.sub.next(&self.client).await?;
		Ok((Block::new(self.client.clone(), info.hash), info))
	}

	pub fn set_follow(&mut self, best_block: bool) {
		self.sub.set_follow(best_block);
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct TransactionSub<T: HasHeader + Decode> {
	client: Client,
	sub: Sub,
	opts: BlockExtOptionsSimple,
	_phantom: PhantomData<T>,
}

impl<T: HasHeader + Decode> TransactionSub<T> {
	pub fn new(client: Client, opts: BlockExtOptionsSimple) -> Self {
		Self { client, sub: Sub::new(), opts, _phantom: Default::default() }
	}

	pub async fn next(&mut self) -> Result<(Vec<BlockTransaction<T>>, BlockRef), crate::Error> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let mut block = BlockWithTx::new(self.client.clone(), info.hash);
			block.set_retry_on_error(self.sub.retry_on_error());

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

	pub fn set_opts(&mut self, value: BlockExtOptionsSimple) {
		self.opts = value;
	}

	pub fn set_follow(&mut self, best_block: bool) {
		self.sub.set_follow(best_block);
	}

	pub fn set_block_height(&mut self, value: u32) {
		self.sub.set_block_height(value);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct ExtrinsicSub<T: HasHeader + Decode> {
	client: Client,
	sub: Sub,
	opts: BlockExtOptionsSimple,
	_phantom: PhantomData<T>,
}

impl<T: HasHeader + Decode> ExtrinsicSub<T> {
	pub fn new(client: Client, opts: BlockExtOptionsSimple) -> Self {
		Self { client, sub: Sub::new(), opts, _phantom: Default::default() }
	}

	pub async fn next(&mut self) -> Result<(Vec<BlockExtrinsic<T>>, BlockRef), crate::Error> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let mut block = BlockWithExt::new(self.client.clone(), info.hash);
			block.set_retry_on_error(self.sub.retry_on_error());

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

	pub fn set_follow(&mut self, best_block: bool) {
		self.sub.set_follow(best_block);
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct RawExtrinsicSub {
	client: Client,
	sub: Sub,
	opts: BlockExtOptionsExpanded,
}

impl RawExtrinsicSub {
	pub fn new(client: Client, opts: BlockExtOptionsExpanded) -> Self {
		Self { client, sub: Sub::new(), opts }
	}

	pub async fn next(&mut self) -> Result<(Vec<BlockRawExtrinsic>, BlockRef), crate::Error> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let mut block = BlockWithRawExt::new(self.client.clone(), info.hash);
			block.set_retry_on_error(self.sub.retry_on_error());

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

	pub fn set_follow(&mut self, best_block: bool) {
		self.sub.set_follow(best_block);
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct EventsSub {
	client: Client,
	sub: Sub,
	opts: BlockEventsOptions,
}

impl EventsSub {
	pub fn new(client: Client, opts: BlockEventsOptions) -> Self {
		Self { client, sub: Sub::new(), opts }
	}

	pub async fn next(&mut self) -> Result<Vec<BlockPhaseEvent>, crate::Error> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let block = BlockEvents::new(self.client.clone(), info.hash);
			let events = match block.block(self.opts.clone()).await {
				Ok(x) => x,
				Err(err) => {
					// Revet block height if we fail to fetch events
					self.sub.set_block_height(info.height);
					return Err(err);
				},
			};

			if events.is_empty() {
				continue;
			}

			return Ok(events);
		}
	}

	pub fn set_follow(&mut self, best_block: bool) {
		self.sub.set_follow(best_block);
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct BlockHeaderSub {
	client: Client,
	sub: Sub,
}

impl BlockHeaderSub {
	pub fn new(client: Client) -> Self {
		Self { client, sub: Sub::new() }
	}

	pub async fn next(&mut self) -> Result<Option<AvailHeader>, RpcError> {
		let info = self.sub.next(&self.client).await?;
		let header = match self
			.client
			.rpc()
			.retry_on(Some(self.sub.retry_on_error()), Some(true))
			.block_header(Some(info.hash))
			.await
		{
			Ok(x) => x,
			Err(err) => {
				// Revet block height if we fail to fetch block header
				self.sub.set_block_height(info.height);
				return Err(err);
			},
		};

		Ok(header)
	}

	pub fn set_follow(&mut self, best_block: bool) {
		self.sub.set_follow(best_block);
	}

	pub fn set_block_height(&mut self, block_height: u32) {
		self.sub.set_block_height(block_height);
	}

	pub fn set_pool_rate(&mut self, value: Duration) {
		self.sub.set_pool_rate(value);
	}

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct GrandpaJustificationSub {
	client: Client,
	sub: Sub,
}

impl GrandpaJustificationSub {
	pub fn new(client: Client) -> Self {
		Self { client, sub: Sub::new() }
	}

	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let just = match self
				.client
				.rpc()
				.retry_on(Some(self.sub.retry_on_error()), None)
				.grandpa_block_justification(info.height)
				.await
			{
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

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[derive(Clone)]
pub struct GrandpaJustificationJsonSub {
	client: Client,
	sub: Sub,
}

impl GrandpaJustificationJsonSub {
	pub fn new(client: Client) -> Self {
		Self { client, sub: Sub::new() }
	}

	pub async fn next(&mut self) -> Result<GrandpaJustification, RpcError> {
		loop {
			let info = self.sub.next(&self.client).await?;
			let just = match self
				.client
				.rpc()
				.retry_on(Some(self.sub.retry_on_error()), None)
				.grandpa_block_justification_json(info.height)
				.await
			{
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

	pub fn set_retry_on_error(&mut self, value: bool) {
		self.sub.set_retry_on_error(value);
	}
}

#[cfg(test)]
mod tests {
	use avail_rust_core::{
		avail::data_availability::tx::SubmitData, decoded_transaction::TransactionEncodable,
		grandpa::GrandpaJustification, rpc::SignerPayload,
	};
	use codec::Encode;
	use serde::{Deserialize, Serialize};
	use serde_json::value::RawValue;

	use crate::{
		block::BlockExtOptionsExpanded,
		clients::mock_client::MockClient,
		error::Error,
		prelude::*,
		subscription::{
			ExtrinsicSub, GrandpaJustificationJsonSub, GrandpaJustificationSub, RawExtrinsicSub, TransactionSub,
		},
		subxt_rpcs::RpcClient,
	};

	fn prep_just(value: Option<GrandpaJustification>) -> Box<RawValue> {
		match value.clone() {
			Some(x) => {
				let value = serde_json::to_string(&Some(const_hex::encode(x.encode()))).unwrap();
				RawValue::from_string(value).unwrap()
			},
			None => {
				let value = serde_json::to_string(&value).unwrap();
				RawValue::from_string(value).unwrap()
			},
		}
	}

	fn prep_just_json(value: Option<GrandpaJustification>) -> Box<RawValue> {
		match value.clone() {
			Some(x) => {
				let value = serde_json::to_string(&Some(x)).unwrap();
				RawValue::from_string(value).unwrap()
			},
			None => {
				let value = serde_json::to_string(&value).unwrap();
				RawValue::from_string(value).unwrap()
			},
		}
	}

	fn prep_ext_info(value: Vec<ExtrinsicInformation>) -> Box<RawValue> {
		let value = serde_json::to_string(&value).unwrap();
		RawValue::from_string(value).unwrap()
	}

	#[derive(Debug, Default, Clone, Serialize, Deserialize)]
	struct ExtrinsicInformation {
		// Hex string encoded
		pub encoded: Option<String>,
		pub tx_hash: H256,
		pub tx_index: u32,
		pub pallet_id: u8,
		pub call_id: u8,
		pub signature: Option<SignerPayload>,
	}

	#[tokio::test]
	async fn grandpa_justification_sub_test() -> Result<(), Error> {
		_ = Client::init_tracing(false);
		let (rpc_client, mut commander) = MockClient::new(MAINNET_ENDPOINT);
		let client = Client::new_rpc_client(RpcClient::new(rpc_client)).await?;

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
		let method = "grandpa_blockJustification";
		commander.add_ok(method, prep_just(Some(GrandpaJustification::default()))); // 1
		commander.add_ok(method, prep_just(None)); // 2
		commander.add_ok(method, prep_just(Some(GrandpaJustification::default()))); // 3
		commander.add_err(method, subxt_rpcs::Error::DisconnectedWillReconnect("Error".into())); // 4
		commander.add_ok(method, prep_just(Some(GrandpaJustification::default()))); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(false);
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}

	#[tokio::test]
	async fn grandpa_justification_json_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(MAINNET_ENDPOINT);
		let client = Client::new_rpc_client(RpcClient::new(rpc_client)).await?;

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
		let method = "grandpa_blockJustificationJson";
		commander.add_ok(method, prep_just_json(Some(GrandpaJustification::default()))); // 1
		commander.add_ok(method, prep_just_json(None)); // 2
		commander.add_ok(method, prep_just_json(Some(GrandpaJustification::default()))); // 3
		commander.add_err(method, subxt_rpcs::Error::DisconnectedWillReconnect("Error".into())); // 4
		commander.add_ok(method, prep_just_json(Some(GrandpaJustification::default()))); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(false);
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}

	#[tokio::test]
	async fn extrinsic_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::new_rpc_client(RpcClient::new(rpc_client)).await?;

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

		let method = "system_fetchExtrinsicsV1";
		commander.add_ok(method, prep_ext_info(vec![data.clone()])); // 1
		commander.add_ok(method, prep_ext_info(vec![])); // 2
		commander.add_ok(method, prep_ext_info(vec![data.clone()])); // 3
		commander.add_err(method, subxt_rpcs::Error::DisconnectedWillReconnect("Error".into())); // 4
		commander.add_ok(method, prep_ext_info(vec![data.clone()])); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(false);
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}

	#[tokio::test]
	async fn transaction_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::new_rpc_client(RpcClient::new(rpc_client)).await?;

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

		let method = "system_fetchExtrinsicsV1";
		commander.add_ok(method, prep_ext_info(vec![data.clone()])); // 1
		commander.add_ok(method, prep_ext_info(vec![])); // 2
		commander.add_ok(method, prep_ext_info(vec![data.clone()])); // 3
		commander.add_err(method, subxt_rpcs::Error::DisconnectedWillReconnect("Error".into())); // 4
		commander.add_ok(method, prep_ext_info(vec![data.clone()])); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(false);
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}

	#[tokio::test]
	async fn raw_extrinsic_sub_test() -> Result<(), Error> {
		let (rpc_client, mut commander) = MockClient::new(TURING_ENDPOINT);
		let client = Client::new_rpc_client(RpcClient::new(rpc_client)).await?;

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

		let method = "system_fetchExtrinsicsV1";
		commander.add_ok(method, prep_ext_info(vec![data.clone()])); // 1
		commander.add_ok(method, prep_ext_info(vec![])); // 2
		commander.add_ok(method, prep_ext_info(vec![data.clone()])); // 3
		commander.add_err(method, subxt_rpcs::Error::DisconnectedWillReconnect("Error".into())); // 4
		commander.add_ok(method, prep_ext_info(vec![data.clone()])); // 4

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 2);
		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		sub.set_retry_on_error(false);
		let _ = sub.next().await.expect_err("Expect Error");
		assert_eq!(sub.sub.as_finalized().next_block_height, 4);

		let _ = sub.next().await?;
		assert_eq!(sub.sub.as_finalized().next_block_height, 5);

		Ok(())
	}
}
