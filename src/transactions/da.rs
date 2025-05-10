use crate::{avail, client::Client, SubmittableTransaction};

#[derive(Clone)]
pub struct DataAvailability {
	pub client: Client,
}

impl DataAvailability {
	pub fn create_application_key(&self, key: Vec<u8>) -> SubmittableTransaction {
		let call = avail::tx().data_availability().create_application_key(key);
		SubmittableTransaction::new(self.client.clone(), call)
	}

	pub fn submit_data(&self, data: Vec<u8>) -> SubmittableTransaction {
		let call = avail::tx().data_availability().submit_data(data);
		SubmittableTransaction::new(self.client.clone(), call)
	}
}
