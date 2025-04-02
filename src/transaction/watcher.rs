use log::warn;
use primitive_types::H256;
use std::{sync::Arc, time::Duration};
use subxt::backend::StreamOf;

use super::{logger::Logger, utils::TransactionExecutionError, TransactionDetails};
use crate::{block::EventRecords, ABlock, Client, ClientMode, WaitFor};

type Stream = StreamOf<Result<ABlock, subxt::Error>>;

pub const DEFAULT_BLOCK_COUNT_TIMEOUT: u32 = 6;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WatcherMode {
	WS,
	HTTP,
	TxStateRPC,
}

#[derive(Clone)]
pub struct Watcher {
	client: Client,
	options: WatcherOptions,
}

#[derive(Clone)]
pub struct WatcherOptions {
	pub wait_for: WaitFor,
	pub block_count_timeout: Option<u32>,
	pub block_height_timeout: Option<u32>,
	// Applicable only to WatcherMode::HTTP. Default os 3 seconds
	pub block_fetch_interval: Duration,
	pub tx_hash: H256,
	pub mode: WatcherMode,
	pub logger: Arc<Logger>,
}

impl WatcherOptions {
	pub fn new(tx_hash: H256, mode: WatcherMode) -> Self {
		WatcherOptions {
			wait_for: WaitFor::BlockInclusion,
			block_count_timeout: None,
			block_height_timeout: None,
			block_fetch_interval: Duration::from_secs(3),
			tx_hash,
			mode,
			logger: Logger::new(H256::default(), false),
		}
	}

	async fn calculate_block_height_timeout(&self, client: &Client) -> Result<u32, TransactionExecutionError> {
		if let Some(height) = self.block_height_timeout {
			return Ok(height);
		}

		let count = self.block_count_timeout.unwrap_or(DEFAULT_BLOCK_COUNT_TIMEOUT);
		let current_height = client.best_block_number().await?;

		Ok(current_height + count)
	}
}

impl Watcher {
	pub fn new(client: Client, tx_hash: H256) -> Self {
		let opt = client.get_options();
		let mode = if opt.tx_state_rpc_enabled {
			WatcherMode::TxStateRPC
		} else if opt.mode == ClientMode::WS {
			WatcherMode::WS
		} else {
			WatcherMode::HTTP
		};

		let options = WatcherOptions::new(tx_hash, mode);
		Self { client, options }
	}

	pub fn set_options<F: Fn(&mut WatcherOptions)>(&mut self, f: F) {
		f(&mut self.options)
	}

	pub async fn run(&self) -> Result<Option<TransactionDetails>, TransactionExecutionError> {
		match self.options.mode {
			WatcherMode::HTTP => HTTPWatcher::run(&self.client, &self.options).await,
			WatcherMode::WS => WSWatcher::run(&self.client, &self.options).await,
			WatcherMode::TxStateRPC => TxStateRPCWatcher::run(&self.client, &self.options).await,
		}
	}
}

pub struct WSWatcher {}
impl WSWatcher {
	pub async fn run(
		client: &Client,
		options: &WatcherOptions,
	) -> Result<Option<TransactionDetails>, TransactionExecutionError> {
		let logger = options.logger.clone();

		let mut stream = match options.wait_for == WaitFor::BlockInclusion {
			true => client.blocks().subscribe_all().await,
			false => client.blocks().subscribe_finalized().await,
		}?;

		let block_height_timeout = options.calculate_block_height_timeout(client).await?;
		if logger.is_enabled() {
			let best = client.best_block_number().await?;
			let finalized = client.finalized_block_number().await?;
			logger.log_watcher_run(options, best, finalized, block_height_timeout);
		}

		loop {
			let block = WSWatcher::fetch_next_block(&mut stream).await?;
			logger.log_watcher_new_block(&block);

			if let Some(tx_details) = find_transaction(client, &block, &options.tx_hash).await? {
				logger.log_watcher_tx_found(&tx_details);
				return Ok(Some(tx_details));
			}

			if block.number() >= block_height_timeout {
				logger.log_watcher_stop();
				return Ok(None);
			}
		}
	}

	async fn fetch_next_block(stream: &mut Stream) -> Result<ABlock, TransactionExecutionError> {
		loop {
			let Some(block) = stream.next().await else {
				return Err(TransactionExecutionError::BlockStreamFailure);
			};

			let block = match block {
				Ok(b) => b,
				Err(e) => {
					if e.is_disconnected_will_reconnect() {
						warn!("The RPC connection was lost and we may have missed a few blocks");
						continue;
					}

					return Err(TransactionExecutionError::SubxtError(e));
				},
			};

			return Ok(block);
		}
	}
}

pub struct HTTPBlockFetch {
	client: Client,
	options: WatcherOptions,
	current_block_height: u32,
	current_block_hash: H256,
}

impl HTTPBlockFetch {
	pub fn new(client: Client, options: WatcherOptions, block_height: u32) -> Self {
		Self {
			client,
			options,
			current_block_height: block_height,
			current_block_hash: H256::default(),
		}
	}

	pub async fn fetch(&mut self) -> Result<ABlock, TransactionExecutionError> {
		loop {
			let block_height = match self.options.wait_for {
				WaitFor::BlockInclusion => self.client.best_block_number().await?,
				WaitFor::BlockFinalization => self.client.finalized_block_number().await?,
			};

			if self.options.wait_for == WaitFor::BlockInclusion {
				// We are in front
				if self.current_block_height > block_height {
					tokio::time::sleep(self.options.block_fetch_interval).await;
					continue;
				}

				let block_hash = self.client.block_hash(self.current_block_height).await?;

				// We are lagging behind
				if block_height > self.current_block_height {
					self.current_block_hash = block_hash;
					self.current_block_height += 1;

					return Ok(self.client.block_at(block_hash).await?);
				}

				// We are at same block height
				if self.current_block_hash.eq(&block_hash) {
					tokio::time::sleep(self.options.block_fetch_interval).await;
					continue;
				}

				self.current_block_hash = block_hash;
				return Ok(self.client.block_at(block_hash).await?);
			} else {
				if self.current_block_height > block_height {
					tokio::time::sleep(self.options.block_fetch_interval).await;
					continue;
				}

				let block_hash = self.client.block_hash(self.current_block_height).await?;
				self.current_block_height += 1;

				return Ok(self.client.block_at(block_hash).await?);
			}
		}
	}
}

pub struct HTTPWatcher {}
impl HTTPWatcher {
	pub async fn run(
		client: &Client,
		options: &WatcherOptions,
	) -> Result<Option<TransactionDetails>, TransactionExecutionError> {
		let logger = options.logger.clone();

		let block_height = match options.wait_for {
			WaitFor::BlockInclusion => client.best_block_number().await?,
			WaitFor::BlockFinalization => client.finalized_block_number().await?,
		};

		let block_height_timeout = options.calculate_block_height_timeout(client).await?;

		if logger.is_enabled() {
			let best = client.best_block_number().await?;
			let finalized = client.finalized_block_number().await?;
			logger.log_watcher_run(options, best, finalized, block_height_timeout);
		}

		let mut block_fetcher = HTTPBlockFetch::new(client.clone(), options.clone(), block_height);

		loop {
			let block = block_fetcher.fetch().await?;

			if let Some(tx_details) = find_transaction(client, &block, &options.tx_hash).await? {
				logger.log_watcher_tx_found(&tx_details);
				return Ok(Some(tx_details));
			}

			if block.number() >= block_height_timeout {
				logger.log_watcher_stop();
				return Ok(None);
			}
		}
	}
}

pub struct TxStateRPCWatcher {}
impl TxStateRPCWatcher {
	pub async fn run(
		client: &Client,
		options: &WatcherOptions,
	) -> Result<Option<TransactionDetails>, TransactionExecutionError> {
		let logger = options.logger.clone();

		let block_height_timeout = options.calculate_block_height_timeout(client).await?;
		if logger.is_enabled() {
			let best = client.best_block_number().await?;
			let finalized = client.finalized_block_number().await?;
			logger.log_watcher_run(options, best, finalized, block_height_timeout);
		}

		let finalized = options.wait_for == WaitFor::BlockFinalization;
		let mut current_block_hash: H256 = H256::zero();
		loop {
			current_block_hash = TxStateRPCWatcher::wait_for_new_block(client, options, &current_block_hash).await?;
			logger.log_watcher_new_block_hash(&current_block_hash);

			let mut result = client.transaction_state(&options.tx_hash, finalized).await?;
			if result.is_empty() {
				tokio::time::sleep(Duration::from_secs(2)).await;
				result = client.transaction_state(&options.tx_hash, finalized).await?;
			}
			result.sort_by(|a, b| b.block_height.cmp(&a.block_height));

			if let Some(state) = result.first() {
				let events = client.event_records(state.block_hash.clone()).await.unwrap_or_default();
				let details = TransactionDetails::new(
					client.clone(),
					events,
					state.tx_hash,
					state.tx_index,
					state.block_hash,
					state.block_height,
				);
				logger.log_watcher_tx_found(&details);
				return Ok(Some(details));
			}

			let block_height = client.block_number(current_block_hash.clone()).await?;
			if block_height >= block_height_timeout {
				logger.log_watcher_stop();
				return Ok(None);
			}
		}
	}

	async fn wait_for_new_block(
		client: &Client,
		options: &WatcherOptions,
		current_block_hash: &H256,
	) -> Result<H256, TransactionExecutionError> {
		loop {
			let block_hash = match options.wait_for {
				WaitFor::BlockInclusion => client.best_block_hash().await?,
				WaitFor::BlockFinalization => client.finalized_block_hash().await?,
			};

			if current_block_hash.eq(&block_hash) {
				tokio::time::sleep(options.block_fetch_interval).await;
				continue;
			}

			return Ok(block_hash);
		}
	}
}

pub async fn find_transaction(
	client: &Client,
	block: &ABlock,
	tx_hash: &H256,
) -> Result<Option<TransactionDetails>, subxt::Error> {
	let transactions = block.extrinsics().await?;
	let tx_found = transactions.iter().find(|e| e.hash() == *tx_hash);
	let Some(ext_details) = tx_found else {
		return Ok(None);
	};

	let events = match ext_details.events().await.ok() {
		Some(x) => EventRecords::new_ext(x),
		None => None,
	};

	let value = TransactionDetails::new(
		client.clone(),
		events,
		*tx_hash,
		ext_details.index(),
		block.hash(),
		block.number(),
	);

	Ok(Some(value))
}
