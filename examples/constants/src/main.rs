//! This example showcases the following actions:
//! - Reading constants from metadata
//!

use avail_rust_client::prelude::*;
use codec::Decode;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	let client = Client::new(LOCAL_ENDPOINT).await?;
	let metadata = client.online_client().metadata();

	// All Pallet names
	for pallet in metadata.pallets() {
		println!("Pallet: {}", pallet.name());
	}
	let pallet = metadata.pallet_by_name("DataAvailability").expect("Should be there");

	// All Constant names
	for constant in pallet.constants() {
		println!("Constant: {}", constant.name())
	}

	let constant = pallet.constant_by_name("MaxAppDataLength").expect("Should be there");
	println!(
		"Constant Name: {}, Constant Value: {}, Constant Docs: {:?}",
		constant.name(),
		u32::decode(&mut constant.value())?,
		constant.docs()
	);

	Ok(())
}
