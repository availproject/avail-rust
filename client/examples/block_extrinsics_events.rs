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

	printout_events("encoded", encoded.events(client.clone()).await?);
	printout_events("extrinsics", extrinsics.events(client.clone()).await?);

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

/*
	Expected Output:

	encoded:
		Index: 2, Pallet ID: 6, Variant ID: 8, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 3, Pallet ID: 29, Variant ID: 1, Data Length: 132, Phase: ApplyExtrinsic(1)
		Index: 4, Pallet ID: 6, Variant ID: 7, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 5, Pallet ID: 6, Variant ID: 7, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 6, Pallet ID: 6, Variant ID: 7, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 7, Pallet ID: 7, Variant ID: 0, Data Length: 132, Phase: ApplyExtrinsic(1)
		Index: 8, Pallet ID: 0, Variant ID: 0, Data Length: 36, Phase: ApplyExtrinsic(1)
	extrinsics:
		Index: 2, Pallet ID: 6, Variant ID: 8, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 3, Pallet ID: 29, Variant ID: 1, Data Length: 132, Phase: ApplyExtrinsic(1)
		Index: 4, Pallet ID: 6, Variant ID: 7, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 5, Pallet ID: 6, Variant ID: 7, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 6, Pallet ID: 6, Variant ID: 7, Data Length: 100, Phase: ApplyExtrinsic(1)
		Index: 7, Pallet ID: 7, Variant ID: 0, Data Length: 132, Phase: ApplyExtrinsic(1)
		Index: 8, Pallet ID: 0, Variant ID: 0, Data Length: 36, Phase: ApplyExtrinsic(1)
*/
