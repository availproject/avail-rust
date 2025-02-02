use crate::{
	api_dev::api::data_availability::calls::types::{
		create_application_key::Key, submit_data::Data,
	},
	avail, Client, Transaction,
};

pub type SubmitDataCall = avail::data_availability::calls::types::SubmitData;
pub type CreateApplicationKeyCall = avail::data_availability::calls::types::CreateApplicationKey;

#[derive(Clone)]
pub struct DataAvailability {
	pub(crate) client: Client,
}

impl DataAvailability {
	pub fn submit_data(&self, data: Vec<u8>) -> Transaction<SubmitDataCall> {
		let data = Data { 0: data };
		let payload = avail::tx().data_availability().submit_data(data);
		Transaction::new(self.client.clone(), payload)
	}

	pub fn create_application_key(&self, key: Vec<u8>) -> Transaction<CreateApplicationKeyCall> {
		let key = Key { 0: key };
		let payload = avail::tx().data_availability().create_application_key(key);
		Transaction::new(self.client.clone(), payload)
	}
}
