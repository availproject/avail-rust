use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// 1 Submittable
	let submittable = client.tx().data_availability().submit_data(2, "My data");
	let call_hash = submittable.call_hash();
	let estimated_fee = submittable.estimate_call_fees(None).await?;
	let weight = submittable.call_info(None).await?.weight;
	println!(
		"Call Hash: {:?}, Estimated Fee: {}, Weight: {}",
		call_hash,
		estimated_fee.final_fee(),
		weight.ref_time
	);

	// 2 Submitting
	let signer = Keypair::from_str("//Bob")?;
	let submitted = submittable.sign_and_submit(&signer, Options::new()).await?;
	println!(
		"Ext Hash: {:?}, Account Id: {}, Nonce: {}",
		submitted.ext_hash, submitted.account_id, submitted.options.nonce
	);

	// 3 Getting Extrinsic Receipt
	let receipt = submitted.receipt(false).await?.expect("Should be included");
	println!("Block State: {}", receipt.block_state().await?);
	println!(
		"Block Height: {}, Block Hash: {:?}, Ext Hash: {:?}, Ext Index: {}",
		receipt.block_height, receipt.block_hash, receipt.ext_hash, receipt.ext_index
	);

	// 4 Fetching Extrinsic Events
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

	// 5 Fetching Extrinsic itself
	let ext = receipt.extrinsic::<avail::data_availability::tx::SubmitData>().await?;
	println!("Data: {:?}", String::from_utf8(ext.call.data).unwrap());

	Ok(())
}

/*
	Expected Output:

	Call Hash: 0x3ef6074c55575799d2e1cfca27b36e95e8c713acf9ae101f14d8e886db8fd6c4, Estimated Fee: 124424722263925292, Weight: 32423250
	Ext Hash: 0x19aa142e2abdfd146f76b13637fafd74a1355655d66b0a23ce8617e280ada595, Account Id: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Nonce: 503, App Id: 2
	Block State: Finalized
	Block Height: 2495956, Block Hash: 0xb3aff50e0b10abe76dd0091ba2d02aced9f678fdef36ca1c54990758ef0117fb, Ext Hash: 0x19aa142e2abdfd146f76b13637fafd74a1355655d66b0a23ce8617e280ada595, Ext Index: 1
	Is Successful: true, Who: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Data Hash: 0xe91967d389a73e279a69db56c95ed7f37b09d737f12ee0b483f35554522da01f
	Data: "My data"
*/
