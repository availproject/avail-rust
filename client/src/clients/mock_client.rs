use crate::{
	clients::ReqwestClient,
	ext::subxt_rpcs::{self, RpcClientT},
};
use avail_rust_core::grandpa::GrandpaJustification;
use codec::Encode;
use serde_json::value::RawValue;
use std::sync::{Arc, Mutex};

#[cfg(test)]
use avail_rust_core::rpc::system::fetch_extrinsics::ExtrinsicInformation;

#[derive(Clone)]
pub struct MockClient {
	org: ReqwestClient,
	commander: Arc<Mutex<CommandManager>>,
}

impl MockClient {
	pub fn new(endpoint: &str) -> (Self, CommandManagerHelper) {
		let org = ReqwestClient::new(endpoint);
		let commander = Arc::new(Mutex::new(CommandManager::default()));
		let wrapper = CommandManagerHelper { 0: commander.clone() };
		(Self { org, commander }, wrapper)
	}
}

impl RpcClientT for MockClient {
	fn request_raw<'a>(
		&'a self,
		method: &'a str,
		params: Option<Box<RawValue>>,
	) -> subxt_rpcs::client::RawRpcFuture<'a, Box<RawValue>> {
		{
			let mut commander = self.commander.lock().unwrap();
			if let Some(value) = commander.find(method) {
				//println!("Found Mock value: Method: {}", method);
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

	pub fn add_ok(&mut self, method: impl Into<String>, value: Box<RawValue>) {
		self.list.push((method.into(), Ok(value)));
	}

	pub fn add_err(&mut self, method: impl Into<String>, value: subxt_rpcs::Error) {
		self.list.push((method.into(), Err(value)));
	}
}

pub struct CommandManagerHelper(pub Arc<Mutex<CommandManager>>);
impl CommandManagerHelper {
	pub fn add_ok(&mut self, method: impl Into<String>, value: Box<RawValue>) {
		let mut lock = self.0.lock().unwrap();
		lock.add_ok(method, value);
	}

	pub fn add_err(&mut self, method: impl Into<String>, value: subxt_rpcs::Error) {
		let mut lock = self.0.lock().unwrap();
		lock.add_err(method, value);
	}

	pub fn justification_ok(&mut self, value: Option<GrandpaJustification>) {
		let value = match value.clone() {
			Some(x) => {
				let value = serde_json::to_string(&Some(const_hex::encode(x.encode()))).unwrap();
				RawValue::from_string(value).unwrap()
			},
			None => {
				let value = serde_json::to_string(&value).unwrap();
				RawValue::from_string(value).unwrap()
			},
		};
		self.add_ok("grandpa_blockJustification", value);
	}

	pub fn justification_err(&mut self, value: Option<subxt_rpcs::Error>) {
		let value = value.unwrap_or_else(|| subxt_rpcs::Error::DisconnectedWillReconnect("Error".into()));
		self.add_err("grandpa_blockJustification", value);
	}

	pub fn justification_json_ok(&mut self, value: Option<GrandpaJustification>) {
		let value = match value.clone() {
			Some(x) => {
				let value = serde_json::to_string(&Some(x)).unwrap();
				RawValue::from_string(value).unwrap()
			},
			None => {
				let value = serde_json::to_string(&value).unwrap();
				RawValue::from_string(value).unwrap()
			},
		};
		self.add_ok("grandpa_blockJustificationJson", value);
	}

	pub fn justification_json_err(&mut self, value: Option<subxt_rpcs::Error>) {
		let value = value.unwrap_or_else(|| subxt_rpcs::Error::DisconnectedWillReconnect("Error".into()));
		self.add_err("grandpa_blockJustificationJson", value);
	}

	pub fn extrinsics_ok(&mut self, value: Vec<ExtrinsicInformation>) {
		let value = serde_json::to_string(&value).unwrap();
		let value = RawValue::from_string(value).unwrap();
		self.add_ok("system_fetchExtrinsicsV1", value);
	}

	pub fn extrinsics_err(&mut self, value: Option<subxt_rpcs::Error>) {
		let value = value.unwrap_or_else(|| subxt_rpcs::Error::DisconnectedWillReconnect("Error".into()));
		self.add_err("system_fetchExtrinsicsV1", value);
	}
}
