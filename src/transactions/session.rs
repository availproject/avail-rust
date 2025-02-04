use crate::{avail, avail::runtime_types::da_runtime::primitives::SessionKeys, Client, Transaction};

pub type SetKeysCall = avail::session::calls::types::SetKeys;

#[derive(Clone)]
pub struct Session {
	pub(crate) client: Client,
}

impl Session {
	pub fn set_keys(&self, keys: SessionKeys) -> Transaction<SetKeysCall> {
		let payload = avail::tx().session().set_keys(keys, vec![]);
		Transaction::new(self.client.clone(), payload)
	}
}
