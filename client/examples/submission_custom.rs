use avail_rust_client::prelude::*;

// Custom Extrinsic
// Implementing HasHeader, codec::Decode and coded::Encode is
// enough to create a custom extrinsic
#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomExtrinsic {
	pub data: Vec<u8>,
}
impl HasHeader for CustomExtrinsic {
	const HEADER_INDEX: (u8, u8) = (29, 1);
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let custom = CustomExtrinsic { data: vec![66, 66, 77, 77] };
	let submittable = SubmittableTransaction::from_encodable(client.clone(), custom);
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

	Ext Hash: 0x179084dce1218db4906a17a87ed3f98f013f48fc0ef5861a45ac3b20e6a88f43
	Block State: Finalized
	Block Height: 2503947, Block Hash: 0xa8419ebc1882bcbb48cd99d11c6ed05428fd046cb1f2d7f920368c1dd3b12ce8, Ext Hash: 0x179084dce1218db4906a17a87ed3f98f013f48fc0ef5861a45ac3b20e6a88f43, Ext Index: 1
	Is Successful: true, Who: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Data Hash: 0x1f48d925000ca3b121efee2aebb6adc503aacf91ba856ff239ff33181d573d20
	Data: "BBMM"
*/
