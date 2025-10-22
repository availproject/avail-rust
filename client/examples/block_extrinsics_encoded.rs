use avail_rust_client::{
	block::{BlockEncodedExtrinsic, extrinsic_options::Options},
	prelude::*,
};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let block = client.block(2470159);
	let query = block.encoded();

	let count = query.count(Options::new().app_id(246)).await?;
	let exists = query.exists(Options::new().app_id(100)).await?;

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
	println!("Ext Index: {}, Ext Call Len: {}, App ID: {:?}", bext.ext_index(), bext.call.len(), bext.app_id());
}
