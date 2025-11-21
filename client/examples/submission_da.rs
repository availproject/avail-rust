use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let submittable = client.tx().data_availability().submit_data(2, "My data");
	let signer = Keypair::from_str("//Bob")?;
	let submitted = submittable.sign_and_submit(&signer, Options::new()).await?;
	println!("Ext Hash: {:?}", submitted.ext_hash,);

	// Getting Extrinsic Receipt
	let receipt = submitted.receipt(false).await?.expect("Should be included");
	println!("Block State: {}", receipt.block_state().await?);
	println!(
		"Block Height: {}, Block Hash: {:?}, Ext Hash: {:?}, Ext Index: {}",
		receipt.block_height, receipt.block_hash, receipt.ext_hash, receipt.ext_index
	);

	// Fetching Extrinsic Events
	let events = receipt.events().await?;
	let event = events
		.first::<avail::data_availability::events::DataSubmitted>()
		.expect("Should be there");
	println!(
		"Is Successful: {}, Who: {}, Data Hash: {:?}",
		events.is_extrinsic_success_present(),
		event.who,
		event.data_hash
	);

	// Fetching Extrinsic itself
	let ext = receipt.extrinsic::<avail::data_availability::tx::SubmitData>().await?;
	println!("Data: {:?}", String::from_utf8(ext.call.data).unwrap());

	Ok(())
}

/*
	Expected Output:

	Ext Hash: 0x773823d76fb6fce38763bbe35b0da67540f248672ba306ad4ed337e0c929eb1a
	Block State: Finalized
	Block Height: 2495971, Block Hash: 0x95eea46c8f4da87196dc35ac720a2a060298fffb31e755169d1dc9ef614f53ba, Ext Hash: 0x773823d76fb6fce38763bbe35b0da67540f248672ba306ad4ed337e0c929eb1a, Ext Index: 1
	Is Successful: true, Who: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Data Hash: 0xe91967d389a73e279a69db56c95ed7f37b09d737f12ee0b483f35554522da01f
	Data: "My data"
*/
