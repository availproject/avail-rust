use crate::{AvailHeader, Client, platform::sleep};
use avail_rust_core::{H256, grandpa::GrandpaJustification, rpc::BlockWithJustifications};
use std::time::Duration;

#[derive(Clone)]
pub enum Subscriber {
	BestBlock {
		poll_rate: Duration,
		current_block_height: u32,
		block_processed: Vec<H256>,
		stored_height: Option<u32>,
	},
	FinalizedBlock {
		poll_rate: Duration,
		next_block_height: u32,
		stored_height: Option<u32>,
	},
}

impl Subscriber {
	pub fn new_best_block(poll_rate_milli: u64, block_height: u32) -> Self {
		Self::BestBlock {
			poll_rate: Duration::from_millis(poll_rate_milli),
			current_block_height: block_height,
			block_processed: Vec::new(),
			stored_height: None,
		}
	}

	pub fn new_finalized_block(poll_rate_milli: u64, block_height: u32) -> Self {
		Self::FinalizedBlock {
			poll_rate: Duration::from_millis(poll_rate_milli),
			next_block_height: block_height,
			stored_height: None,
		}
	}

	pub async fn run(&mut self, client: Client) -> Result<Option<(u32, H256)>, avail_rust_core::Error> {
		match self {
			Subscriber::BestBlock {
				poll_rate,
				current_block_height,
				block_processed,
				stored_height,
			} => {
				return Self::run_best_block(client, *poll_rate, current_block_height, block_processed, stored_height)
					.await;
			},
			Subscriber::FinalizedBlock { poll_rate, next_block_height, stored_height } => {
				return Self::run_finalized(client, *poll_rate, next_block_height, stored_height).await;
			},
		}
	}

	pub fn current_block_height(&self) -> u32 {
		match self {
			Subscriber::BestBlock {
				poll_rate: _,
				current_block_height,
				block_processed: _,
				stored_height: _,
			} => *current_block_height,
			Subscriber::FinalizedBlock { poll_rate: _, next_block_height, stored_height: _ } => {
				if *next_block_height > 0 {
					*next_block_height - 1
				} else {
					*next_block_height
				}
			},
		}
	}

	pub fn stored_height(&self) -> Option<u32> {
		match self {
			Subscriber::BestBlock {
				poll_rate: _,
				current_block_height: _,
				block_processed: _,
				stored_height,
			} => stored_height.clone(),
			Subscriber::FinalizedBlock { poll_rate: _, next_block_height: _, stored_height } => stored_height.clone(),
		}
	}

	async fn run_best_block(
		client: Client,
		poll_rate: Duration,
		current_block_height: &mut u32,
		block_processed: &mut Vec<H256>,
		stored_height: &mut Option<u32>,
	) -> Result<Option<(u32, H256)>, avail_rust_core::Error> {
		let block_height = *current_block_height;
		let res =
			Self::fetch_best_block_height(client, block_height, block_processed, poll_rate, stored_height).await?;
		if let Some(res) = &res {
			*current_block_height = res.0;
			if res.0 > block_height {
				block_processed.clear();
			}
			block_processed.push(res.1);
		} else {
			*current_block_height += 1;
			block_processed.clear();
		}

		Ok(res)
	}

	async fn run_finalized(
		client: Client,
		poll_rate: Duration,
		next_block_height: &mut u32,
		stored_height: &mut Option<u32>,
	) -> Result<Option<(u32, H256)>, avail_rust_core::Error> {
		let block_height = *next_block_height;

		if stored_height.is_none() {
			let new_height = client.finalized_block_height().await?;
			*stored_height = Some(new_height);
		}

		if stored_height.is_some_and(|stored_h| stored_h > block_height) {
			let block_hash = client.block_hash(block_height).await?;
			*next_block_height += 1;
			return Ok(block_hash.map(|x| (block_height, x)));
		}

		let block_hash = loop {
			let new_height = client.finalized_block_height().await?;
			if block_height > new_height {
				sleep(poll_rate).await;
				continue;
			}

			break client.block_hash_with_retries(block_height).await?;
		};

		*next_block_height += 1;

		Ok(block_hash.map(|x| (block_height, x)))
	}

	async fn fetch_best_block_height(
		client: Client,
		current_block_height: u32,
		block_processed: &[H256],
		poll_rate: Duration,
		stored_height: &mut Option<u32>,
	) -> Result<Option<(u32, H256)>, avail_rust_core::Error> {
		if stored_height.is_none() {
			let new_height = client.best_block_height().await?;
			*stored_height = Some(new_height);
		}

		if stored_height.is_some_and(|stored_h| stored_h > current_block_height) {
			let mut block_height = current_block_height;
			if !block_processed.is_empty() {
				block_height += 1;
			}
			let Some(block_hash) = client.block_hash(block_height).await? else {
				return Ok(None);
			};
			return Ok(Some((block_height, block_hash)));
		}

		loop {
			let best_block_hash = client.best_block_hash().await?;
			if block_processed.contains(&best_block_hash) {
				sleep(poll_rate).await;
				continue;
			}

			let Some(best_block_height) = client.block_height_with_retries(best_block_hash).await? else {
				return Ok(None);
			};

			let is_ahead_of_current_block = best_block_height > current_block_height;
			let is_next_block = best_block_height == (current_block_height + 1);
			let is_current_block = best_block_height == current_block_height;
			let no_block_processed_yet = block_processed.is_empty();

			if is_ahead_of_current_block {
				if no_block_processed_yet {
					let Some(block_hash) = client.block_hash_with_retries(current_block_height).await? else {
						return Ok(None);
					};
					return Ok(Some((current_block_height, block_hash)));
				}

				if is_next_block {
					return Ok(Some((best_block_height, best_block_hash)));
				}

				let next_block_height = current_block_height + 1;
				let Some(next_block_hash) = client.block_hash_with_retries(next_block_height).await? else {
					return Ok(None);
				};

				return Ok(Some((next_block_height, next_block_hash)));
			}

			if is_current_block && !block_processed.contains(&best_block_hash) {
				return Ok(Some((best_block_height, best_block_hash)));
			}

			sleep(poll_rate).await;
			continue;
		}
	}
}

#[derive(Clone)]
pub struct HeaderSubscription {
	client: Client,
	sub: Subscriber,
}

impl HeaderSubscription {
	pub fn new(client: Client, sub: Subscriber) -> Self {
		Self { client, sub }
	}

	pub async fn next(&mut self) -> Result<Option<AvailHeader>, avail_rust_core::Error> {
		let block_info = self.sub.run(self.client.clone()).await?;

		let Some((block_height, block_hash)) = block_info else {
			return Ok(None);
		};

		if let Some(stored_height) = self.sub.stored_height() {
			if stored_height > block_height {
				return self.client.block_header(block_hash).await;
			}
		}

		self.client.block_header_with_retries(block_hash).await
	}

	pub fn stored_height(&self) -> Option<u32> {
		self.sub.stored_height()
	}

	pub fn current_block_height(&self) -> u32 {
		self.sub.current_block_height()
	}
}

#[derive(Clone)]
pub struct BlockSubscription {
	client: Client,
	sub: Subscriber,
}

impl BlockSubscription {
	pub fn new(client: Client, sub: Subscriber) -> Self {
		Self { client, sub }
	}

	pub async fn next(&mut self) -> Result<Option<BlockWithJustifications>, avail_rust_core::Error> {
		let block_info = self.sub.run(self.client.clone()).await?;

		let Some((block_height, block_hash)) = block_info else {
			return Ok(None);
		};

		if let Some(stored_height) = self.sub.stored_height() {
			if stored_height > block_height {
				return self.client.block(block_hash).await;
			}
		}

		self.client.block_with_retries(block_hash).await
	}

	pub fn stored_height(&self) -> Option<u32> {
		self.sub.stored_height()
	}

	pub fn current_block_height(&self) -> u32 {
		self.sub.current_block_height()
	}
}

// This one is a bit different.
#[derive(Clone)]
pub struct GrandpaJustificationsSubscription {
	client: Client,
	block_height: u32,
	poll_rate: Duration,
	stored_height: Option<u32>,
}

impl GrandpaJustificationsSubscription {
	pub fn new(client: Client, poll_rate_ms: u64, block_height: u32) -> Self {
		Self {
			client,
			block_height,
			poll_rate: Duration::from_millis(poll_rate_ms),
			stored_height: None,
		}
	}

	pub async fn next(&mut self) -> Result<(GrandpaJustification, u32), avail_rust_core::Error> {
		loop {
			let stored_height = if let Some(height) = self.stored_height.as_ref() {
				*height
			} else {
				let height = self.client.finalized_block_height().await?;
				self.stored_height = Some(height);
				height
			};

			if self.block_height > stored_height {
				let finalized_block_height = self.client.finalized_block_height().await?;
				if self.block_height > finalized_block_height {
					sleep(self.poll_rate).await;
					continue;
				}
			}

			let height = self.block_height;
			let grandpa_justification = self.client.rpc_api().grandpa_block_justification(height).await?;

			self.block_height += 1;
			let Some(justification) = grandpa_justification else {
				continue;
			};

			return Ok((justification, height));
		}
	}

	pub fn current_block_height(&self) -> u32 {
		if self.block_height > 0 {
			self.block_height - 1
		} else {
			self.block_height
		}
	}
}

#[cfg(test)]
pub mod test {
	use std::{
		sync::{Arc, Mutex},
		time::Duration,
	};

	use avail_rust_core::{H256, ext::subxt_rpcs::RpcClient};

	use crate::{
		Client,
		clients::reqwest_client::{ReqwestClient, testable::*},
		constants,
		subscription::Subscriber,
	};

	/* 	#[tokio::test]
	async fn todo_name_finalized_block_test() {
		let rv = ReturnValues::new();
		let rv = Arc::new(Mutex::new(rv));
		let rpc_client = ReqwestClient::new_mocked(rv.clone());
		let rpc_client = RpcClient::new(rpc_client);
		let client = Client::new_rpc_client(rpc_client).await.unwrap();

		let mut todo_name = super::TodoName::FinalizedBlock { next_block_height: 0 };
		let res1 = todo_name.run(client.clone(), Duration::from_millis(100)).await.unwrap();
		ReturnValues::lock_new_block(&rv, 1, H256::default());
		let res2 = todo_name.run(client, Duration::from_millis(100)).await.unwrap();

		dbg!(res1);
		dbg!(res2);
	}
	 */

	#[tokio::test]
	async fn todoname_test2() {
		let client = Client::new(constants::TURING_ENDPOINT).await.unwrap();
		let mut header_sub = client.subscription_block_header(Subscriber::new_finalized_block(19222232, 0));
		let mut i = 0;
		while let Ok(Some(header)) = header_sub.next().await {
			dbg!(header.number);

			i += 1;
			if i > 1000 {
				break;
			}
		}
	}
}
