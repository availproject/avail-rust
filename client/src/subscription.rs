use crate::{AvailHeader, Client, platform::sleep};
use avail_rust_core::{
	H256,
	rpc::{BlockJustification, BlockWithJustifications},
};
use std::time::Duration;

#[derive(Clone)]
pub enum Subscriber {
	BestBlock {
		poll_rate: Duration,
		current_block_height: u32,
		block_processed: Vec<H256>,
	},
	FinalizedBlock {
		poll_rate: Duration,
		next_block_height: u32,
	},
}

impl Subscriber {
	pub fn new_best_block(poll_rate_milli: u64, block_height: u32) -> Self {
		Self::BestBlock {
			poll_rate: Duration::from_millis(poll_rate_milli),
			current_block_height: block_height,
			block_processed: Vec::new(),
		}
	}

	pub fn new_finalized_block(poll_rate_milli: u64, block_height: u32) -> Self {
		Self::FinalizedBlock {
			poll_rate: Duration::from_millis(poll_rate_milli),
			next_block_height: block_height,
		}
	}

	pub async fn run(&mut self, client: Client) -> Result<Option<(u32, H256)>, avail_rust_core::Error> {
		match self {
			Subscriber::BestBlock {
				poll_rate,
				current_block_height,
				block_processed,
			} => {
				return Self::run_best_block(client, *poll_rate, current_block_height, block_processed).await;
			},
			Subscriber::FinalizedBlock {
				poll_rate,
				next_block_height,
			} => {
				return Self::run_finalized(client, *poll_rate, next_block_height).await;
			},
		}
	}

	pub fn current_block_height(&self) -> u32 {
		match self {
			Subscriber::BestBlock {
				poll_rate: _,
				current_block_height,
				block_processed: _,
			} => *current_block_height,
			Subscriber::FinalizedBlock {
				poll_rate: _,
				next_block_height,
			} => {
				if *next_block_height > 0 {
					*next_block_height - 1
				} else {
					*next_block_height
				}
			},
		}
	}

	async fn run_best_block(
		client: Client,
		poll_rate: Duration,
		current_block_height: &mut u32,
		block_processed: &mut Vec<H256>,
	) -> Result<Option<(u32, H256)>, avail_rust_core::Error> {
		let block_height = *current_block_height;
		let res = Self::fetch_best_block_height(client, block_height, block_processed, poll_rate).await?;
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
	) -> Result<Option<(u32, H256)>, avail_rust_core::Error> {
		let block_height = *next_block_height;
		let res = Self::fetch_finalized_block_height(client, block_height, poll_rate).await?;
		*next_block_height += 1;

		Ok(res.map(|x| (block_height, x)))
	}

	pub async fn fetch_best_block_height(
		client: Client,
		current_block_height: u32,
		block_processed: &[H256],
		poll_rate: Duration,
	) -> Result<Option<(u32, H256)>, avail_rust_core::Error> {
		loop {
			let best_block_hash = client.best_block_hash().await?;
			if block_processed.contains(&best_block_hash) {
				sleep(poll_rate).await;
				continue;
			}

			let Some(best_block_height) = client.block_height(best_block_hash).await? else {
				return Ok(None);
			};

			let is_ahead_of_current_block = best_block_height > current_block_height;
			let is_next_block = best_block_height == (current_block_height + 1);
			let is_current_block = best_block_height == current_block_height;
			let no_block_processed_yet = block_processed.is_empty();

			if is_ahead_of_current_block {
				if no_block_processed_yet {
					let Some(block_hash) = client.block_hash(current_block_height).await? else {
						return Ok(None);
					};
					return Ok(Some((current_block_height, block_hash)));
				}

				if is_next_block {
					return Ok(Some((best_block_height, best_block_hash)));
				}

				let next_block_height = current_block_height + 1;
				let Some(next_block_hash) = client.block_hash(next_block_height).await? else {
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

	pub async fn fetch_finalized_block_height(
		client: Client,
		target_block_height: u32,
		poll_rate: Duration,
	) -> Result<Option<H256>, avail_rust_core::Error> {
		loop {
			let finalized_block_height = client.finalized_block_height().await?;
			if target_block_height > finalized_block_height {
				sleep(poll_rate).await;
				continue;
			}

			return client.block_hash(target_block_height).await;
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

		let Some((_block_height, block_hash)) = block_info else {
			return Ok(None);
		};

		self.client.block_header(block_hash).await
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

		let Some((_block_height, block_hash)) = block_info else {
			return Ok(None);
		};

		self.client.block(block_hash).await
	}

	pub fn current_block_height(&self) -> u32 {
		self.sub.current_block_height()
	}
}

#[derive(Clone)]
pub struct JustificationsSubscription {
	client: Client,
	sub: Subscriber,
}

impl JustificationsSubscription {
	pub fn new(client: Client, kind: Subscriber) -> Self {
		Self { client, sub: kind }
	}

	pub async fn next(&mut self) -> Result<(Vec<BlockJustification>, (u32, H256)), avail_rust_core::Error> {
		loop {
			let Some(block_position) = self.sub.run(self.client.clone()).await? else {
				let err = "Cannot fetch block justifications as block is not available.".into();
				return Err(err);
			};

			let Some(block) = self.client.block(block_position.1).await? else {
				let err = "Cannot fetch block justifications as block is not available.".into();
				return Err(err);
			};

			let Some(justifications) = block.justifications else {
				continue;
			};

			return Ok((justifications, block_position));
		}
	}

	pub fn current_block_height(&self) -> u32 {
		self.sub.current_block_height()
	}
}

#[derive(Clone)]
pub struct GrandpaJustificationsSubscription {
	client: Client,
	sub: Subscriber,
}

impl GrandpaJustificationsSubscription {
	pub fn new(client: Client, kind: Subscriber) -> Self {
		Self { client, sub: kind }
	}

	pub async fn next(&mut self) -> Result<(BlockJustification, (u32, H256)), avail_rust_core::Error> {
		loop {
			let Some(block_position) = self.sub.run(self.client.clone()).await? else {
				let err = "Cannot fetch block justifications as block is not available.".into();
				return Err(err);
			};

			let Some(block) = self.client.block(block_position.1).await? else {
				let err = "Cannot fetch block justifications as block is not available.".into();
				return Err(err);
			};

			let Some(justifications) = block.justifications else {
				continue;
			};

			let Some(justification) = justifications.into_iter().find(|x| x.0.cmp(b"FRNK").is_eq()) else {
				continue;
			};

			return Ok((justification, block_position));
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
