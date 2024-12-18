use avail_rust::error::ClientError;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	data_availability_create_key().await?;
	balances_transfer_keep_alive().await?;
	identity_set_identity().await?;

	// Payload
	Ok(())
}

async fn data_availability_create_key() -> Result<(), ClientError> {
	let key = String::from("My Data").into_bytes();
	let key = avail_rust::BoundedVec(key);
	let _payload = avail_rust::avail::tx()
		.data_availability()
		.create_application_key(key);
	Ok(())
}

async fn balances_transfer_keep_alive() -> Result<(), ClientError> {
	let dest = avail_rust::account::account_id_from_str(
		"5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
	)?; // Bob
	let value = avail_rust::SDK::one_avail();
	let _payload = avail_rust::avail::tx()
		.balances()
		.transfer_keep_alive(dest.into(), value);

	Ok(())
}

async fn identity_set_identity() -> Result<(), ClientError> {
	use avail_rust::avail::{
		identity::calls::types::set_identity::Info,
		runtime_types::{
			bounded_collections::bounded_vec::BoundedVec, pallet_identity::types::Data,
		},
	};

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

	let _payload = avail_rust::avail::tx().identity().set_identity(info);

	Ok(())
}
