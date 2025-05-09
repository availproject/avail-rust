#[cfg(feature = "subxt_metadata")]
use crate::avail::data_availability::calls::types::{create_application_key::Key, submit_data::Data};

use crate::{avail, client::Client, SubmittableTransaction};

pub type SubmitDataCall = avail::data_availability::calls::types::SubmitData;
pub type CreateApplicationKeyCall = avail::data_availability::calls::types::CreateApplicationKey;

#[derive(Clone)]
pub struct DataAvailability {
	pub client: Client,
}

impl DataAvailability {
	pub fn create_application_key(&self, key: Vec<u8>) -> SubmittableTransaction<CreateApplicationKeyCall> {
		#[cfg(feature = "subxt_metadata")]
		let payload = avail::tx().data_availability().create_application_key(Key { 0: key });
		#[cfg(not(feature = "subxt_metadata"))]
		let payload = avail::tx().data_availability().create_application_key(key);

		SubmittableTransaction::new(self.client.clone(), payload)
	}

	pub fn submit_data(&self, data: Vec<u8>) -> SubmittableTransaction<SubmitDataCall> {
		#[cfg(feature = "subxt_metadata")]
		let payload = avail::tx().data_availability().submit_data(Data { 0: data });
		#[cfg(not(feature = "subxt_metadata"))]
		let payload = avail::tx().data_availability().submit_data(data);

		SubmittableTransaction::new(self.client.clone(), payload)
	}
}
