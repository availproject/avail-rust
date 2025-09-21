use avail_rust_client::{
	clients::ReqwestClient,
	ext::subxt_rpcs::{self, RpcClientT},
};
use serde_json::value::RawValue;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MockClient {
	org: ReqwestClient,
	commands: Arc<Mutex<CommandManager>>,
}

impl MockClient {
	pub fn new(endpoint: &str) -> Self {
		let org = ReqwestClient::new(endpoint);
		let commands = Arc::new(Mutex::new(CommandManager::default()));
		Self { org, commands }
	}

	pub fn on_next(&self, method: impl Into<String>, value: Box<RawValue>) {}
}

impl RpcClientT for MockClient {
	fn request_raw<'a>(
		&'a self,
		method: &'a str,
		params: Option<Box<RawValue>>,
	) -> subxt_rpcs::client::RawRpcFuture<'a, Box<RawValue>> {
		{
			let mut commands = self.commands.lock().unwrap();
			if let Some(value) = commands.find(method) {
				return Box::pin(async move { value });
			}
		}

		self.org.request_raw(method, params)
	}

	fn subscribe_raw<'a>(
		&'a self,
		sub: &'a str,
		params: Option<Box<RawValue>>,
		unsub: &'a str,
	) -> subxt_rpcs::client::RawRpcFuture<'a, subxt_rpcs::client::RawRpcSubscription> {
		self.org.subscribe_raw(sub, params, unsub)
	}
}

#[derive(Debug, Default)]
pub struct CommandManager {
	list: Vec<(String, Result<Box<RawValue>, subxt_rpcs::Error>)>,
}

impl CommandManager {
	pub fn find(&mut self, method: &str) -> Option<Result<Box<RawValue>, subxt_rpcs::Error>> {
		let pos = self.list.iter().position(|x| x.0.as_str() == method);
		let Some(pos) = pos else {
			return None;
		};
		let value = self.list.remove(pos);
		Some(value.1)
	}
}
