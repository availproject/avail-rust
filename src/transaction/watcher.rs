use std::{sync::Arc, time::Duration};

use log::warn;
use primitive_types::H256;
use subxt::backend::StreamOf;

use super::{logger::Logger, utils::TransactionExecutionError, TransactionDetails};
use crate::{block::EventRecords, sdk::ClientMode, ABlock, Client, WaitFor};

type Stream = StreamOf<Result<ABlock, subxt::Error>>;

pub const DEFAULT_BLOCK_COUNT_TIMEOUT: u32 = 5;

#[derive(Clone)]
pub struct Watcher {
	client: Client,
	tx_hash: H256,
	wait_for: WaitFor,
	block_count_timeout: Option<u32>,
	block_height_timeout: Option<u32>,
	logger: Arc<Logger>,
	block_fetch_interval: Duration,
	client_mode: ClientMode,
}

impl Watcher {
	pub fn new(client: Client, tx_hash: H256) -> Self {
		let client_mode = client.mode;
		Self {
			client,
			tx_hash,
			wait_for: WaitFor::BlockInclusion,
			block_count_timeout: None,
			block_height_timeout: None,
			logger: Logger::new(H256::default(), false),
			block_fetch_interval: Duration::from_secs(3),
			client_mode,
		}
	}
	pub fn client_mode(mut self, value: ClientMode) -> Self {
		self.client_mode = value;
		self
	}

	pub fn block_fetch_interval(mut self, value: Duration) -> Self {
		self.block_fetch_interval = value;
		self
	}

	pub fn logger(mut self, value: Arc<Logger>) -> Self {
		self.logger = value;
		self
	}

	pub fn wait_for(mut self, value: WaitFor) -> Self {
		self.wait_for = value;
		self
	}

	pub fn tx_hash(mut self, value: H256) -> Self {
		self.tx_hash = value;
		self
	}

	pub fn block_count_timeout(mut self, value: u32) -> Self {
		self.block_count_timeout = Some(value);
		self
	}

	pub fn block_height_timeout(mut self, value: u32) -> Self {
		self.block_height_timeout = Some(value);
		self
	}

	pub async fn run(&self) -> Result<Option<TransactionDetails>, TransactionExecutionError> {
		match self.client_mode {
			ClientMode::HTTP => self.http_run().await,
			ClientMode::WS => self.ws_run().await,
		}
	}

	pub async fn ws_run(&self) -> Result<Option<TransactionDetails>, TransactionExecutionError> {
		let mut stream = self.pick_stream().await?;
		let block_height_timeout = self.calculate_block_height_timeout().await?;
		self.logger
			.log_watcher_run(self.wait_for, block_height_timeout, self.client.mode);

		loop {
			let block = self.ws_fetch_next_block(&mut stream).await?;
			self.logger.log_watcher_new_block(&block);

			if let Some(tx_details) = self.find_transaction(&block).await? {
				self.logger.log_watcher_tx_found(&tx_details);
				return Ok(Some(tx_details));
			}

			if block.number() >= block_height_timeout {
				self.logger.log_watcher_stop();
				return Ok(None);
			}
		}
	}

	pub async fn http_run(&self) -> Result<Option<TransactionDetails>, TransactionExecutionError> {
		let block_height_timeout = self.calculate_block_height_timeout().await?;
		self.logger
			.log_watcher_run(self.wait_for, block_height_timeout, self.client_mode);

		let mut current_block_hash: Option<H256> = None;
		loop {
			let block = self.http_fetch_next_block(&mut current_block_hash).await?;
			self.logger.log_watcher_new_block(&block);

			if let Some(tx_details) = self.find_transaction(&block).await? {
				self.logger.log_watcher_tx_found(&tx_details);
				return Ok(Some(tx_details));
			}

			if block.number() >= block_height_timeout {
				self.logger.log_watcher_stop();
				return Ok(None);
			}
		}
	}

	async fn http_fetch_next_block(
		&self,
		current_block_hash: &mut Option<H256>,
	) -> Result<ABlock, TransactionExecutionError> {
		loop {
			let block_hash = match self.wait_for {
				WaitFor::BlockInclusion => self.client.best_block_hash().await?,
				WaitFor::BlockFinalization => self.client.finalized_block_hash().await?,
			};

			if current_block_hash.is_some_and(|hash| hash.eq(&block_hash)) {
				tokio::time::sleep(self.block_fetch_interval).await;
				continue;
			}

			*current_block_hash = Some(block_hash);

			return Ok(self.client.block_at(block_hash).await?);
		}
	}

	async fn pick_stream(&self) -> Result<Stream, subxt::Error> {
		match self.wait_for == WaitFor::BlockInclusion {
			true => self.client.blocks().subscribe_all().await,
			false => self.client.blocks().subscribe_finalized().await,
		}
	}

	async fn ws_fetch_next_block(&self, stream: &mut Stream) -> Result<ABlock, TransactionExecutionError> {
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

	async fn calculate_block_height_timeout(&self) -> Result<u32, TransactionExecutionError> {
		if let Some(height) = self.block_height_timeout {
			return Ok(height);
		}

		let count = self.block_count_timeout.unwrap_or(DEFAULT_BLOCK_COUNT_TIMEOUT);
		let current_height = self.client.best_block_number().await?;

		Ok(current_height + count)
	}

	async fn find_transaction(&self, block: &ABlock) -> Result<Option<TransactionDetails>, subxt::Error> {
		let transactions = block.extrinsics().await?;
		let tx_found = transactions.iter().find(|e| e.hash() == self.tx_hash);
		let Some(ext_details) = tx_found else {
			return Ok(None);
		};

		let events = match ext_details.events().await.ok() {
			Some(x) => EventRecords::new_ext(x),
			None => None,
		};

		let value = TransactionDetails::new(
			self.client.clone(),
			events,
			self.tx_hash,
			ext_details.index(),
			block.hash(),
			block.number(),
		);

		Ok(Some(value))
	}
}
