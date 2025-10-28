use avail_rust_client::{block::BlockExtrinsic, prelude::*};
use avail_rust_core::avail::{
	data_availability::tx::{CreateApplicationKey, SubmitData},
	timestamp::tx::Set,
};
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
	let timestamp = query.get::<Set>(0).await?.expect("Should be there");
	printout_details(&timestamp);
	println!("Get: Timestamp::Set now: {}", timestamp.call.now);
	println!("");

	// 2
	let first = query
		.first::<SubmitData>(Default::default())
		.await?
		.expect("Should be there");
	printout_details(&first);
	println!("First: DataAvailability::SubmitData data len: {}", first.call.data.len());

	let last = query
		.last::<SubmitData>(Default::default())
		.await?
		.expect("Should be there");
	printout_details(&last);
	println!("Last: DataAvailability::SubmitData data len: {}", last.call.data.len());
	println!("");

	// 3
	let all = query.all::<SubmitData>(Default::default()).await?;
	all.iter().for_each(printout_details);
	println!(
		"Last from All: DataAvailability::SubmitData data len: {}",
		all.last().as_ref().unwrap().call.data.len()
	);
	println!("");

	Ok(())
}

pub fn printout_details<T: HasHeader + Decode>(bext: &BlockExtrinsic<T>) {
	println!(
		"Ext Index: {}, Ext Call Pallet ID: {}, Ext Call Variant ID: {}, App ID: {:?}",
		bext.ext_index(),
		T::HEADER_INDEX.0,
		T::HEADER_INDEX.1,
		bext.app_id()
	);
}

/*
	Expected Output:

	Block 2470159 has 6 DataAvailability::SubmitData extrinsics
	Does Block 2470159 have DataAvailability::CreateApplicationKey extrinsics? false

	Ext Index: 0, Ext Call Pallet ID: 3, Ext Call Variant ID: 0, App ID: None
	Get: Timestamp::Set now: 1761144640000

	Ext Index: 1, Ext Call Pallet ID: 29, Ext Call Variant ID: 1, App ID: Some(1)
	First: DataAvailability::SubmitData data len: 5
	Ext Index: 6, Ext Call Pallet ID: 29, Ext Call Variant ID: 1, App ID: Some(246)
	Last: DataAvailability::SubmitData data len: 1578

	Ext Index: 1, Ext Call Pallet ID: 29, Ext Call Variant ID: 1, App ID: Some(1)
	Ext Index: 2, Ext Call Pallet ID: 29, Ext Call Variant ID: 1, App ID: Some(2)
	Ext Index: 3, Ext Call Pallet ID: 29, Ext Call Variant ID: 1, App ID: Some(246)
	Ext Index: 4, Ext Call Pallet ID: 29, Ext Call Variant ID: 1, App ID: Some(246)
	Ext Index: 5, Ext Call Pallet ID: 29, Ext Call Variant ID: 1, App ID: Some(246)
	Ext Index: 6, Ext Call Pallet ID: 29, Ext Call Variant ID: 1, App ID: Some(246)
	Last from All: DataAvailability::SubmitData data len: 1578
*/
