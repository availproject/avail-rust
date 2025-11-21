use avail_rust_client::{
	block::{BlockEncodedExtrinsic, extrinsic_options::Options},
	prelude::*,
};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let block = client.block(2470159);
	let query = block.encoded();

	let count = query.count(Options::new()).await?;
	let exists = query.exists(Options::new()).await?;

	println!("Block 2470159 has {} extrinsics with app id 246", count);
	println!("Does Block 2470159 have extrinsics with app id 100? {}", exists);
	println!("");

	// 1
	let bext = query.get(0).await?.expect("Should be there");
	printout_details(&bext);
	let call = avail::timestamp::tx::Set::from_call(bext.call).unwrap();
	println!("Get: Timestamp::Set now: {}", call.now);
	println!("");

	// 2
	let options = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX);
	let first = query.first(options.clone()).await?.expect("Should be there");
	printout_details(&first);
	let call = avail::data_availability::tx::SubmitData::from_call(first.call).unwrap();
	println!("First: DataAvailability::SubmitData data len: {}", call.data.len());

	let last = query.last(options).await?.expect("Should be there");
	printout_details(&last);
	let call = avail::data_availability::tx::SubmitData::from_call(last.call).unwrap();
	println!("Last: DataAvailability::SubmitData data len: {}", call.data.len());
	println!("");

	// 3
	let options = Options::new().filter(avail::data_availability::tx::SubmitData::HEADER_INDEX);
	let all = query.all(options.clone()).await?;
	all.iter().for_each(printout_details);
	let call = avail::data_availability::tx::SubmitData::from_call(all.last().map(|x| &x.call).unwrap()).unwrap();
	println!("Last from All: DataAvailability::SubmitData data len: {}", call.data.len());
	println!("");

	Ok(())
}

pub fn printout_details(bext: &BlockEncodedExtrinsic) {
	println!("Ext Index: {}, Ext Call Len: {}", bext.ext_index(), bext.call.len());
}

/*
	Expected Output:

	Block 2470159 has 4 extrinsics with app id 246
	Does Block 2470159 have extrinsics with app id 100? false

	Ext Index: 0, Ext Call Len: 9, App ID: None
	Get: Timestamp::Set now: 1761144640000

	Ext Index: 1, Ext Call Len: 8, App ID: Some(1)
	First: DataAvailability::SubmitData data len: 5
	Ext Index: 6, Ext Call Len: 1582, App ID: Some(246)
	Last: DataAvailability::SubmitData data len: 1578

	Ext Index: 1, Ext Call Len: 8, App ID: Some(1)
	Ext Index: 2, Ext Call Len: 8, App ID: Some(2)
	Ext Index: 3, Ext Call Len: 154, App ID: Some(246)
	Ext Index: 4, Ext Call Len: 375, App ID: Some(246)
	Ext Index: 5, Ext Call Len: 630, App ID: Some(246)
	Ext Index: 6, Ext Call Len: 1582, App ID: Some(246)
	Last from All: DataAvailability::SubmitData data len: 1578
*/
