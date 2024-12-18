use avail_rust::{account::account_id_from_str, error::ClientError};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	data_availability_create_key().await?;
	balances_transfer_keep_alive().await?;
	identity_set_identity().await?;

	// Payload
	Ok(())
}

async fn data_availability_create_key() -> Result<(), ClientError> {
	use avail_rust::{
		avail::runtime_types::bounded_collections::bounded_vec::BoundedVec,
		subxt::{blocks::StaticExtrinsic, ext::subxt_core::tx::payload::StaticPayload},
	};

	use avail_rust::avail::data_availability::calls::types::CreateApplicationKey;
	let pallet_name = CreateApplicationKey::PALLET;
	let call_name = CreateApplicationKey::CALL;

	let key = String::from("My Data").into_bytes();
	let key = BoundedVec(key);
	let call_data = CreateApplicationKey { key };

	let _payload = StaticPayload::new(pallet_name, call_name, call_data);

	Ok(())
}

async fn balances_transfer_keep_alive() -> Result<(), ClientError> {
	use avail_rust::{
		subxt::{blocks::StaticExtrinsic, ext::subxt_core::tx::payload::StaticPayload},
		SDK,
	};

	use avail_rust::avail::balances::calls::types::TransferKeepAlive;
	let pallet_name = TransferKeepAlive::PALLET;
	let call_name = TransferKeepAlive::CALL;

	let dest = account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?; // Bob
	let value = SDK::one_avail();
	let call_data = TransferKeepAlive {
		dest: dest.into(),
		value,
	};

	let _payload = StaticPayload::new(pallet_name, call_name, call_data);

	Ok(())
}

async fn identity_set_identity() -> Result<(), ClientError> {
	use avail_rust::subxt::{blocks::StaticExtrinsic, ext::subxt_core::tx::payload::StaticPayload};

	use avail_rust::avail::{
		identity::calls::types::{set_identity::Info, SetIdentity},
		runtime_types::{
			bounded_collections::bounded_vec::BoundedVec, pallet_identity::types::Data,
		},
	};
	let pallet_name = SetIdentity::PALLET;
	let call_name = SetIdentity::CALL;

	let display_name: [u8; 7] = String::from("My Name").into_bytes().try_into().unwrap();
	let info = Info {
		additional: BoundedVec(vec![(Data::None, Data::None)]),
		display: Data::Raw7(display_name),
		legal: Data::None,
		web: Data::None,
		riot: Data::None,
		email: Data::None,
		pgp_fingerprint: None,
		image: Data::None,
		twitter: Data::None,
	};

	let call_data = SetIdentity {
		info: Box::new(info),
	};

	let _payload = StaticPayload::new(pallet_name, call_name, call_data);

	Ok(())
}
