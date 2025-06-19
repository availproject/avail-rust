use avail_rust_core::{rpc::BlockWithJustifications, H256};
use std::{
	fmt::Debug,
	sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct CacheClient {
	inner: Arc<RwLock<CacheClientInner>>,
}

impl CacheClient {
	pub fn new() -> Self {
		let signed_blocks = CachedSingedBlocks::new(3);
		let inner = CacheClientInner { signed_blocks };
		Self {
			inner: Arc::new(RwLock::new(inner)),
		}
	}

	pub fn find_signed_block(&self, block_hash: H256) -> Option<Arc<BlockWithJustifications>> {
		let lock = self.inner.read().expect("Should not be poisoned");
		lock.signed_blocks.find(block_hash)
	}

	pub fn push_signed_block(&self, value: (H256, Arc<BlockWithJustifications>)) {
		let mut lock = self.inner.write().expect("Should not be poisoned");
		lock.signed_blocks.push(value)
	}

	pub fn toggle_caching(&self, value: bool) {
		let mut lock = self.inner.write().expect("Should not be poisoned");
		lock.signed_blocks.enabled = value;
		lock.signed_blocks.blocks.truncate(0);
	}
}

#[derive(Clone)]
struct CacheClientInner {
	pub signed_blocks: CachedSingedBlocks,
}

impl Debug for CacheClient {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("CacheClient").field("last_fetched_block", &"").finish()
	}
}

#[derive(Clone)]
struct CachedSingedBlocks {
	blocks: Vec<Option<(H256, Arc<BlockWithJustifications>)>>,
	ptr: usize,
	enabled: bool,
}

impl CachedSingedBlocks {
	pub fn new(max_cap: usize) -> Self {
		let blocks = Vec::with_capacity(max_cap);
		Self {
			blocks,
			ptr: 0,
			enabled: true,
		}
	}

	pub fn find(&self, block_hash: H256) -> Option<Arc<BlockWithJustifications>> {
		if !self.enabled {
			return None;
		}

		for (hash, block) in self.blocks.iter().flatten() {
			if *hash == block_hash {
				return Some(block.clone());
			}
		}

		None
	}

	pub fn push(&mut self, value: (H256, Arc<BlockWithJustifications>)) {
		if !self.enabled {
			return;
		}

		if self.ptr == self.blocks.len() {
			self.blocks.push(Some(value));
		} else {
			self.blocks[self.ptr] = Some(value);
		}

		self.ptr += 1;
		if self.ptr >= self.blocks.capacity() {
			self.ptr = 0;
		}
	}
}
