use async_trait::async_trait;
use std::{future::Future, pin::Pin, time::Duration};

#[cfg(feature = "native")]
pub type AsyncTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[cfg(feature = "wasm")]
pub type AsyncTask = Pin<Box<dyn Future<Output = ()> + 'static>>;

#[cfg(all(feature = "native", feature = "tokio"))]
pub use tokio::sync::mpsc::{Receiver as AsyncReceiver, Sender as AsyncSender};

#[cfg(all(feature = "native", feature = "tokio"))]
pub fn async_channel<T>(capacity: usize) -> (AsyncSender<T>, AsyncReceiver<T>) {
	tokio::sync::mpsc::channel(capacity)
}

#[cfg(any(
	feature = "wasm",
	all(feature = "async-channel", feature = "native", not(feature = "tokio")),
))]
pub use async_channel::{Receiver as AsyncReceiver, Sender as AsyncSender};

#[cfg(any(
	feature = "wasm",
	all(feature = "async-channel", feature = "native", not(feature = "tokio")),
))]
pub fn async_channel<T>(capacity: usize) -> (AsyncSender<T>, AsyncReceiver<T>) {
	::async_channel::bounded(capacity)
}

#[cfg(all(feature = "native", feature = "tokio"))]
pub async fn async_recv<T>(receiver: &mut AsyncReceiver<T>) -> Option<T> {
	receiver.recv().await
}

#[cfg(any(
	feature = "wasm",
	all(feature = "async-channel", feature = "native", not(feature = "tokio")),
))]
pub async fn async_recv<T>(receiver: &mut AsyncReceiver<T>) -> Option<T> {
	receiver.recv().await.ok()
}

#[async_trait]
pub trait AsyncOp: Send + Sync {
	async fn sleep(&self, duration: Duration);
	fn spawn(&self, task: AsyncTask);
}

#[cfg(any(all(feature = "native", any(feature = "tokio", feature = "smol")), feature = "wasm"))]
#[derive(Clone)]
pub struct StandardAsyncOp;

#[cfg(any(all(feature = "native", any(feature = "tokio", feature = "smol")), feature = "wasm"))]
#[async_trait]
impl AsyncOp for StandardAsyncOp {
	#[cfg(all(feature = "native", feature = "tokio"))]
	async fn sleep(&self, duration: Duration) {
		tokio::time::sleep(duration).await;
	}

	#[cfg(all(feature = "native", feature = "smol"))]
	async fn sleep(&self, duration: Duration) {
		smol::Timer::after(duration).await;
	}

	#[cfg(feature = "wasm")]
	async fn sleep(&self, duration: Duration) {
		wasmtimer::tokio::sleep(duration).await;
	}

	#[cfg(all(feature = "native", feature = "tokio"))]
	fn spawn(&self, task: AsyncTask) {
		tokio::spawn(task);
	}

	#[cfg(all(feature = "native", feature = "smol"))]
	fn spawn(&self, task: AsyncTask) {
		let _ = smol::spawn(task);
	}

	#[cfg(feature = "wasm")]
	fn spawn(&self, task: AsyncTask) {
		wasm_bindgen_futures::spawn_local(task);
	}
}
