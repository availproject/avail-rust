use avail_rust_client::{block::BlockSignedExtrinsic, prelude::*};
use avail_rust_core::avail::data_availability::tx::{CreateApplicationKey, SubmitData};
use codec::Decode;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let block = client.block(2470159);
	let query = block.extrinsics();

	let count = query.count::<SubmitData>(Default::default()).await?;
	let exists = query.exists::<CreateApplicationKey>(Default::default()).await?;

	println!("Block 2470159 has {} DataAvailability::SubmitData extrinsics", count);
	println!("Does Block 2470159 have DataAvailability::CreateApplicationKey extrinsics? {}", exists);
	println!("");

	// 1
	let submit_data = query
		.get::<SubmitData>(1)
		.await?
		.expect("Should be there")
		.as_signed()
		.unwrap();
	printout_details(&submit_data);
	println!("Get: DataAvailability::SubmitData data len: {}", submit_data.call.data.len());
	println!("");

	// 2
	let first = query
		.first::<SubmitData>(Default::default())
		.await?
		.expect("Should be there")
		.as_signed()
		.unwrap();
	printout_details(&first);
	println!("First: DataAvailability::SubmitData data len: {}", first.call.data.len());

	let last = query
		.last::<SubmitData>(Default::default())
		.await?
		.expect("Should be there")
		.as_signed()
		.unwrap();
	printout_details(&last);
	println!("Last: DataAvailability::SubmitData data len: {}", last.call.data.len());
	println!("");

	// 3
	let all = query.all::<SubmitData>(Default::default()).await?;
	all.iter()
		.filter_map(|x| x.clone().as_signed().ok())
		.for_each(|x| printout_details(&x));
	println!(
		"Last from All: DataAvailability::SubmitData data len: {}",
		all.last().as_ref().unwrap().call.data.len()
	);
	println!("");

	Ok(())
}

pub fn printout_details<T: HasHeader + Decode>(bext: &BlockSignedExtrinsic<T>) {
	println!(
		"Ext Index: {}, Ext Call Pallet ID: {}, Ext Call Variant ID: {}, App ID: {:?}",
		bext.ext_index(),
		T::HEADER_INDEX.0,
		T::HEADER_INDEX.1,
		bext.app_id()
	);
}
