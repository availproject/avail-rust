use avail_rust_client::prelude::*;
use avail_rust_core::HasHeader;
use codec::{Decode, Encode};

// Custom call types
//
// The SDK's transaction API (client.tx().data_availability().submit_data(...)) covers the standard
// Avail pallets, but sometimes you need to submit a call that isn't in the generated bindings.
// Maybe you're targeting a custom pallet or a call variant the SDK doesn't expose yet.
//
// To do this, define a struct that implements HasHeader + Encode + Decode. The HEADER_INDEX
// constant tells the runtime which pallet and call variant this maps to — it's the (pallet_index,
// call_index) pair from the chain's metadata. For example, (29, 1) is DA::submit_data on Avail.
//
// Once you have the struct, pass it to SubmittableTransaction::from_encodable() and the rest of
// the submission flow works exactly the same as with built-in calls.
#[derive(Decode, Encode, PartialEq, Eq)]
pub struct CustomExtrinsic {
	pub data: Vec<u8>,
}

impl HasHeader for CustomExtrinsic {
	const HEADER_INDEX: (u8, u8) = (29, 1);
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::connect(TURING_ENDPOINT).await?;
	let signer = Account::new_from_str("//Bob")?;

	// Build and submit the custom call. From here on it's identical to any other transaction —
	// submit, wait for a receipt, read events.
	let custom = CustomExtrinsic { data: vec![1, 2, 3, 4] };
	let tx = SubmittableTransaction::from_encodable(client.clone(), custom);
	let submitted = tx.submit(&signer, Options::new()).await?;
	println!("Ext Hash: {:?}", submitted.ext_hash);

	let (receipt, events) = submitted.outcome(BlockQueryMode::Finalized).await?;
	println!("Included: height={}", receipt.block_height);

	// We can read events emitted by our call just like any other transaction.
	let event = events
		.first::<avail::data_availability::events::DataSubmitted>()
		.expect("Should be there");
	println!("Data hash: {:?}", event.data_hash);

	Ok(())
}
