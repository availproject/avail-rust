use std::{
	future::Future,
	sync::{atomic::AtomicBool, Arc, Mutex},
	time::Duration,
};

use avail_rust::prelude::*;
use tokio::task::JoinHandle;

type SharedLock<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
	Manual,
	Stream,
}

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;
	let mut indexer = Indexer::new(sdk.clone());
	indexer.run(Kind::Stream);

	// Fetching blocks in procedural way
	let mut sub = indexer.subscribe().await;
	for _ in 0..3 {
		let block = sub.fetch().await;
		println!("Current: Block Height: {}, Block Hash: {:?}", block.height, block.hash)
	}

	// Fetching historical blocks
	sub.block_height -= 100;
	for _ in 0..3 {
		let block = sub.fetch().await;
		println!(
			"Historical: Block Height: {}, Block Hash: {:?}",
			block.height, block.hash
		)
	}

	// Callback
	let mut sub = indexer.callback(callback).await;
	tokio::time::sleep(Duration::from_secs(25)).await;

	sub.shutdown();
	indexer.shutdown();

	tokio::time::sleep(Duration::from_secs(3)).await;

	Ok(())
}

async fn callback(block: IndexedBlock) {
	println!("Callback: Block Height: {}, Block Hash: {:?}", block.height, block.hash)
}

#[derive(Clone)]
struct Indexer {
	block: SharedLock<Option<IndexedBlock>>,
	sdk: Arc<SDK>,
	thread: SharedLock<Option<JoinHandle<()>>>,
}

impl Indexer {
	pub fn new(sdk: SDK) -> Self {
		Self {
			block: Arc::new(Mutex::new(None)),
			sdk: Arc::new(sdk),
			thread: Arc::new(Mutex::new(None)),
		}
	}
	pub fn run(&mut self, kind: Kind) {
		if self.thread.lock().unwrap().is_some() {
			return;
		}

		let block = self.block.clone();
		let sdk = self.sdk.clone();
		let t = tokio::spawn(async move {
			println!("Kind: {:?}", kind);
			match kind {
				Kind::Manual => Self::task_man(block, sdk).await,
				Kind::Stream => Self::task_sub(block, sdk).await,
			};
		});

		self.thread = Arc::new(Mutex::new(Some(t)))
	}

	pub fn shutdown(&mut self) {
		let lock = self.thread.lock().unwrap();
		let Some(t) = lock.as_ref() else {
			return;
		};
		t.abort();
	}

	pub async fn get_block(&self, block_height: u32) -> IndexedBlock {
		loop {
			let block = self.block.lock().unwrap().clone();

			let Some(block) = block else {
				tokio::time::sleep(Duration::from_secs(5)).await;
				continue;
			};

			if block_height > block.height {
				tokio::time::sleep(Duration::from_secs(5)).await;
				continue;
			}

			if block_height == block.height {
				return block;
			}

			let block_hash = self.sdk.client.block_hash(block_height).await.unwrap();
			let block = Block::new(&self.sdk.client, block_hash.clone()).await.unwrap();

			return IndexedBlock {
				height: block_height,
				hash: block_hash,
				block,
			};
		}
	}

	pub async fn subscribe(&self) -> Subscription {
		let block_height = loop {
			let height = {
				let block = self.block.lock().unwrap();
				block.as_ref().map(|x| x.height)
			};

			if height.is_none() {
				tokio::time::sleep(Duration::from_secs(5)).await;
				continue;
			}

			break height.unwrap();
		};

		Subscription::new(block_height, self.clone())
	}

	pub async fn callback<F>(&self, cb: fn(IndexedBlock) -> F) -> Subscription
	where
		F: Future + std::marker::Send + 'static,
	{
		let sub = self.subscribe().await;
		let mut sub2 = sub.clone();
		tokio::spawn(async move {
			loop {
				let block = sub2.fetch().await;
				cb(block).await;
			}
		});

		sub
	}

	async fn task_man(shared_block: SharedLock<Option<IndexedBlock>>, sdk: Arc<SDK>) {
		loop {
			let new_hash = sdk.client.finalized_block_hash().await.unwrap();
			let cur_hash = {
				let block = shared_block.lock().unwrap();
				block.as_ref().map(|x| x.hash)
			};

			if cur_hash.is_some_and(|x| x == new_hash) {
				tokio::time::sleep(Duration::from_secs(15)).await;
				continue;
			}

			let new_block = Block::new(&sdk.client, new_hash).await.unwrap();
			let new_height = sdk.client.block_height(new_hash.clone()).await.unwrap();

			let mut cur_block = shared_block.lock().unwrap();
			*cur_block = Some(IndexedBlock {
				height: new_height,
				hash: new_hash,
				block: new_block,
			})
		}
	}

	async fn task_sub(shared_block: SharedLock<Option<IndexedBlock>>, sdk: Arc<SDK>) {
		let mut stream = sdk.client.blocks().subscribe_finalized().await.unwrap();
		loop {
			let block = stream.next().await.unwrap();
			let block = match block {
				Ok(b) => b,
				Err(e) => {
					if e.is_disconnected_will_reconnect() {
						println!("The RPC connection was lost and we may have missed a few blocks");
						continue;
					}

					panic!("Something is wrong");
				},
			};

			let height = block.number();
			let hash = block.hash();
			let block = Block::from_block(block).await.unwrap();

			let mut cur_block = shared_block.lock().unwrap();
			*cur_block = Some(IndexedBlock { height, hash, block })
		}
	}
}

#[derive(Clone)]
struct Subscription {
	pub indexer: Indexer,
	pub block_height: u32,
	pub shutdown: Arc<AtomicBool>,
}

impl Subscription {
	pub fn new(block_height: u32, indexer: Indexer) -> Self {
		Self {
			indexer,
			block_height,
			shutdown: Arc::new(AtomicBool::new(false)),
		}
	}

	pub async fn fetch(&mut self) -> IndexedBlock {
		let block = self.indexer.get_block(self.block_height).await;
		self.block_height += 1;

		block
	}

	pub fn shutdown(&mut self) {
		self.shutdown.store(false, std::sync::atomic::Ordering::Relaxed);
	}
}

#[derive(Clone)]
struct IndexedBlock {
	height: u32,
	hash: H256,
	block: Block,
}
