use crate::{avail, avail::runtime_types::da_runtime::primitives::SessionKeys, Client, SubmittableTransaction};

pub type SetKeysCall = avail::session::calls::types::SetKeys;

#[derive(Clone)]
pub struct Session {
	pub client: Client,
}

impl Session {
	/// Sets the session key(s) of the function caller to `keys`.
	/// Allows an account to set its session key prior to becoming a validator.
	/// This doesn't take effect until the next session.
	pub fn set_keys(&self, keys: SessionKeys) -> SubmittableTransaction<SetKeysCall> {
		let payload = avail::tx().session().set_keys(keys, vec![]);
		SubmittableTransaction::new(self.client.clone(), payload)
	}
}
