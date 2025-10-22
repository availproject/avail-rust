use avail_rust_client::prelude::*;
use avail_rust_core::rpc::ExtrinsicInfo;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let block = client.block(2470159);
	let query = block.extrinsics();

	// println!("Number of extrinsics in block 2470159: {}", block.);
	// printout_info_details(infos);
	// println!("");

	// 1

	// All
	// let get = query.get(0).await?.expect("Should be there");
	// let first = query.first(Default::default()).await?.expect("Should be there");
	// let last = query.last(Default::default()).await?.expect("Should be there");
	// let all = query.all(Default::default()).await?;

	// First

	// Last

	Ok(())
}

pub fn printout_info_details(infos: Vec<ExtrinsicInfo>) {
	for info in infos {
		println!(
			"Index: {}, Hash: {:?}, Pallet ID: {}, Variant ID: {}, App ID: {:?}, SS58 Address: {:?}, Data: {:?}",
			info.ext_index,
			info.ext_hash,
			info.pallet_id,
			info.variant_id,
			info.signer_payload.as_ref().map(|x| x.app_id),
			info.signer_payload.as_ref().map(|x| x.ss58_address.clone()).flatten(),
			info.data,
		);
	}
}
