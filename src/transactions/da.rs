#[cfg(feature = "subxt")]
use crate::{
	avail,
	avail::data_availability::calls::types::{create_application_key::Key, submit_data::Data},
};

use crate::{client::Client, SubmittableTransaction};
use codec::{Decode, Encode};
use subxt_core::blocks::StaticExtrinsic;
use subxt_core::ext::{scale_decode::DecodeAsType, scale_encode::EncodeAsType};
use subxt_core::tx::payload::DefaultPayload;

#[cfg(feature = "subxt")]
pub type SubmitDataCall = avail::data_availability::calls::types::SubmitData;
#[cfg(feature = "subxt")]
pub type CreateApplicationKeyCall = avail::data_availability::calls::types::CreateApplicationKey;

#[cfg(not(feature = "subxt"))]
pub type SubmitDataCall = SubmitData;
#[cfg(not(feature = "subxt"))]
pub type CreateApplicationKeyCall = CreateApplicationKey;

#[derive(Clone)]
pub struct DataAvailability {
	pub client: Client,
}

impl DataAvailability {
	pub fn create_application_key(&self, key: Vec<u8>) -> SubmittableTransaction<CreateApplicationKeyCall> {
		#[cfg(feature = "subxt")]
		let payload = {
			let key = Key { 0: key };
			avail::tx().data_availability().create_application_key(key)
		};

		#[cfg(not(feature = "subxt"))]
		let payload = payload_crate_application_key(key);

		SubmittableTransaction::new(self.client.clone(), payload)
	}

	pub fn submit_data(&self, data: Vec<u8>) -> SubmittableTransaction<SubmitDataCall> {
		#[cfg(feature = "subxt")]
		let payload = {
			let data = Data { 0: data };
			avail::tx().data_availability().submit_data(data)
		};

		#[cfg(not(feature = "subxt"))]
		let payload = payload_submit_data(data);

		SubmittableTransaction::new(self.client.clone(), payload)
	}
}

#[derive(Decode, Encode, DecodeAsType, EncodeAsType, Clone, Debug, Eq, PartialEq)]
#[codec (crate = codec)]
#[decode_as_type(crate_path = ":: subxt_core :: ext :: scale_decode")]
#[encode_as_type(crate_path = ":: subxt_core :: ext :: scale_encode")]
pub struct CreateApplicationKey {
	pub key: Vec<u8>,
}
impl StaticExtrinsic for CreateApplicationKey {
	const PALLET: &'static str = "DataAvailability";
	const CALL: &'static str = "create_application_key";
}
pub fn payload_crate_application_key(key: Vec<u8>) -> DefaultPayload<CreateApplicationKey> {
	DefaultPayload::<CreateApplicationKey>::new(
		CreateApplicationKey::PALLET,
		CreateApplicationKey::CALL,
		CreateApplicationKey { key },
	)
}

#[derive(Decode, Encode, DecodeAsType, EncodeAsType, Clone, Debug, Eq, PartialEq)]
#[codec (crate = codec)]
#[decode_as_type(crate_path = "subxt_core :: ext :: scale_decode")]
#[encode_as_type(crate_path = "subxt_core :: ext :: scale_encode")]
pub struct SubmitData {
	pub data: Vec<u8>,
}
impl StaticExtrinsic for SubmitData {
	const PALLET: &'static str = "DataAvailability";
	const CALL: &'static str = "submit_data";
}
pub fn payload_submit_data(data: Vec<u8>) -> DefaultPayload<SubmitData> {
	DefaultPayload::<SubmitData>::new(SubmitData::PALLET, SubmitData::CALL, SubmitData { data })
}
