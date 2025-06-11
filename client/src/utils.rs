use client_core::AvailHeader;
use futures::{Stream, StreamExt};
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use crate::Client;

pub struct HeaderSubscription {
	client: Arc<Client>,
	next_block_height: u32,
	poll_rate: Duration,
	use_best_block: bool,
	pending: Option<Pin<Box<dyn Future<Output = Result<Option<AvailHeader>, client_core::Error>> + Send + 'static>>>,
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
