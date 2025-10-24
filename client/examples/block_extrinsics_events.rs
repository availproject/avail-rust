use avail_rust_client::{
	block::{BlockEvents, extrinsic_options::Options},
	prelude::*,
};
use avail_rust_core::avail::data_availability::tx::SubmitData;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let block = client.block(2470159);
	let encoded = block
		.encoded()
		.first(Options::new().filter(SubmitData::HEADER_INDEX))
		.await?
		.expect("Must be there");
	let extrinsics = block
		.extrinsics()
		.first::<SubmitData>(Default::default())
		.await?
		.expect("Must be there");
	let signed = block
		.signed()
		.first::<SubmitData>(Default::default())
		.await?
		.expect("Must be there");

	printout_events("encoded", encoded.events(client.clone()).await?);
	printout_events("extrinsics", extrinsics.events(client.clone()).await?);
	printout_events("signed", signed.events(client.clone()).await?);

	Ok(())
}

fn printout_events(from: &str, events: BlockEvents) {
	println!("{}:", from);
	for event in events.events {
		println!(
			"	Index: {}, Pallet ID: {}, Variant ID: {}, Data Length: {}, Phase: {:?}",
			event.index,
			event.pallet_id,
			event.variant_id,
			event.data.len(),
			event.phase,
		);
	}
}
