use crate::{AvailHeader, Client};
use avail_rust_core::{
	rpc::{BlockJustification, BlockWithJustifications},
	H256,
};
use futures::{Stream, StreamExt};
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct JustificationSubscriptionEntry {
	pub block_height: u32,
	pub block_hash: H256,
	pub justifications: Vec<BlockJustification>,
}

impl JustificationSubscriptionEntry {
	pub fn new(block_height: u32, block_hash: H256, justifications: Vec<BlockJustification>) -> Self {
		Self {
			block_height,
			block_hash,
			justifications,
		}
	}
}

pub struct JustificationSubscription {
	client: Arc<Client>,
	next_block_height: u32,
	poll_rate: Duration,
	use_best_block: bool,
	pending: Option<
		Pin<Box<dyn Future<Output = Result<JustificationSubscriptionEntry, avail_rust_core::Error>> + Send + 'static>>,
	>,
}

impl JustificationSubscription {
	pub fn new(client: Arc<Client>, block_height: u32, poll_rate: Duration, use_best_block: bool) -> Self {
		Self {
			client,
			pending: None,
			poll_rate,
			use_best_block,
			next_block_height: block_height,
		}
	}

	pub async fn next(&mut self) -> Option<JustificationSubscriptionEntry> {
		<Self as StreamExt>::next(self).await
	}
}

impl Stream for JustificationSubscription {
	type Item = JustificationSubscriptionEntry;

	fn poll_next(
		mut self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Option<Self::Item>> {
		let mut fut = match self.pending.take() {
			Some(x) => x,
			None => {
				let client = self.client.clone();
				let mut next_block_height = self.next_block_height;
				let poll_rate = self.poll_rate;
				let use_best_block = self.use_best_block;
				let task = async move {
					loop {
						let height = match use_best_block {
							true => client.best_block_height().await?,
							false => client.finalized_block_height().await?,
						};
						if next_block_height > height {
							tokio::time::sleep(poll_rate).await;
							continue;
						}

						let block_hash = client.block_hash(next_block_height).await?.unwrap();
						let block = client.block(block_hash).await?.unwrap();
						if block.justifications.is_none() {
							next_block_height += 1;
							continue;
						}
						return Ok(JustificationSubscriptionEntry::new(
							next_block_height,
							block_hash,
							block.justifications.unwrap(),
						));
					}
				};

				Box::pin(task)
			},
		};

		match fut.as_mut().poll(cx) {
			std::task::Poll::Ready(Ok(entry)) => {
				self.pending = None;
				self.next_block_height = entry.block_height + 1;
				std::task::Poll::Ready(Some(entry))
			},
			std::task::Poll::Ready(Err(_err)) => {
				self.pending = None;
				self.next_block_height += 1;
				std::task::Poll::Ready(None)
			},
			std::task::Poll::Pending => {
				self.pending = Some(fut);
				std::task::Poll::Pending
			},
		}
	}
}

pub struct BlockHeightSubscription {
	client: Arc<Client>,
	next_block_height: u32,
	poll_rate: Duration,
	use_best_block: bool,
	pending:
		Option<Pin<Box<dyn Future<Output = Result<Option<(u32, H256)>, avail_rust_core::Error>> + Send + 'static>>>,
}

impl BlockHeightSubscription {
	pub fn new(client: Arc<Client>, block_height: u32, poll_rate: Duration, use_best_block: bool) -> Self {
		Self {
			client,
			pending: None,
			poll_rate,
			use_best_block,
			next_block_height: block_height,
		}
	}

	pub async fn next(&mut self) -> Option<(u32, H256)> {
		<Self as StreamExt>::next(self).await
	}
}

impl Stream for BlockHeightSubscription {
	type Item = (u32, H256);

	fn poll_next(
		mut self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Option<Self::Item>> {
		let mut fut = match self.pending.take() {
			Some(x) => x,
			None => {
				let client = self.client.clone();
				let next_block_height = self.next_block_height;
				let poll_rate = self.poll_rate;
				let use_best_block = self.use_best_block;
				let task = async move {
					loop {
						let height = match use_best_block {
							true => client.best_block_height().await?,
							false => client.finalized_block_height().await?,
						};
						if next_block_height > height {
							tokio::time::sleep(poll_rate).await;
							continue;
						}

						return Ok(Some((
							next_block_height,
							client.block_hash(next_block_height).await?.unwrap(),
						)));
					}
				};

				Box::pin(task)
			},
		};

		match fut.as_mut().poll(cx) {
			std::task::Poll::Ready(Ok(block_hash)) => {
				self.pending = None;
				self.next_block_height += 1;
				std::task::Poll::Ready(block_hash)
			},
			std::task::Poll::Ready(Err(_err)) => {
				self.pending = None;
				self.next_block_height += 1;
				std::task::Poll::Ready(None)
			},
			std::task::Poll::Pending => {
				self.pending = Some(fut);
				std::task::Poll::Pending
			},
		}
	}
}

pub struct BlockSubscription {
	client: Arc<Client>,
	next_block_height: u32,
	poll_rate: Duration,
	use_best_block: bool,
	pending: Option<
		Pin<Box<dyn Future<Output = Result<Option<BlockWithJustifications>, avail_rust_core::Error>> + Send + 'static>>,
	>,
}

impl BlockSubscription {
	pub fn new(client: Arc<Client>, block_height: u32, poll_rate: Duration, use_best_block: bool) -> Self {
		Self {
			client,
			pending: None,
			poll_rate,
			use_best_block,
			next_block_height: block_height,
		}
	}

	pub async fn next(&mut self) -> Option<BlockWithJustifications> {
		<Self as StreamExt>::next(self).await
	}
}

impl Stream for BlockSubscription {
	type Item = BlockWithJustifications;

	fn poll_next(
		mut self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Option<Self::Item>> {
		let mut fut = match self.pending.take() {
			Some(x) => x,
			None => {
				let client = self.client.clone();
				let next_block_height = self.next_block_height;
				let poll_rate = self.poll_rate;
				let use_best_block = self.use_best_block;
				let task = async move {
					loop {
						let height = match use_best_block {
							true => client.best_block_height().await?,
							false => client.finalized_block_height().await?,
						};
						if next_block_height > height {
							tokio::time::sleep(poll_rate).await;
							continue;
						}

						let block_hash = client.block_hash(next_block_height).await?.unwrap();
						return client.block(block_hash).await;
					}
				};

				Box::pin(task)
			},
		};

		match fut.as_mut().poll(cx) {
			std::task::Poll::Ready(Ok(block)) => {
				self.pending = None;
				self.next_block_height += 1;
				std::task::Poll::Ready(block)
			},
			std::task::Poll::Ready(Err(_err)) => {
				self.pending = None;
				self.next_block_height += 1;
				std::task::Poll::Ready(None)
			},
			std::task::Poll::Pending => {
				self.pending = Some(fut);
				std::task::Poll::Pending
			},
		}
	}
}

pub struct HeaderSubscription {
	client: Arc<Client>,
	next_block_height: u32,
	poll_rate: Duration,
	use_best_block: bool,
	pending:
		Option<Pin<Box<dyn Future<Output = Result<Option<AvailHeader>, avail_rust_core::Error>> + Send + 'static>>>,
}

impl HeaderSubscription {
	pub fn new(client: Arc<Client>, block_height: u32, poll_rate: Duration, use_best_block: bool) -> Self {
		Self {
			client,
			pending: None,
			poll_rate,
			use_best_block,
			next_block_height: block_height,
		}
	}

	pub async fn next(&mut self) -> Option<AvailHeader> {
		<Self as StreamExt>::next(self).await
	}
}

impl Stream for HeaderSubscription {
	type Item = AvailHeader;

	fn poll_next(
		mut self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Option<Self::Item>> {
		let mut fut = match self.pending.take() {
			Some(x) => x,
			None => {
				let client = self.client.clone();
				let next_block_height = self.next_block_height;
				let poll_rate = self.poll_rate;
				let use_best_block = self.use_best_block;
				let task = async move {
					loop {
						let height = match use_best_block {
							true => client.best_block_height().await?,
							false => client.finalized_block_height().await?,
						};
						if next_block_height > height {
							tokio::time::sleep(poll_rate).await;
							continue;
						}

						let block_hash = client.block_hash(next_block_height).await?.unwrap();
						return client.block_header(block_hash).await;
					}
				};

				Box::pin(task)
			},
		};

		match fut.as_mut().poll(cx) {
			std::task::Poll::Ready(Ok(header)) => {
				self.pending = None;
				self.next_block_height += 1;
				std::task::Poll::Ready(header)
			},
			std::task::Poll::Ready(Err(_err)) => {
				self.pending = None;
				self.next_block_height += 1;
				std::task::Poll::Ready(None)
			},
			std::task::Poll::Pending => {
				self.pending = Some(fut);
				std::task::Poll::Pending
			},
		}
	}
}
