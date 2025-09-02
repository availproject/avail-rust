//! This example showcases the following actions:
//! - Creating custom event
//! - Decoding custom event
//!

use avail_rust_client::prelude::*;

#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomEvent {
	pub who: AccountId,
	pub data_hash: H256,
}
impl HasHeader for CustomEvent {
	const HEADER_INDEX: (u8, u8) = (29, 1);
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let submittable = client.tx().data_availability().submit_data(vec![0, 1, 2]);
	let submitted = submittable.sign_and_submit(&alice(), Options::new(Some(2))).await?;
	let receipt = submitted.receipt(true).await?.expect("Must be there");
	let event_group = receipt.tx_events().await?;
	let event = event_group
		.events
		.iter()
		.find(|x| x.emitted_index == CustomEvent::HEADER_INDEX)
		.expect("Must be there");

	let hex_encoded_event = event.encoded.as_ref().expect("Must be there");
	let event = CustomEvent::decode_hex_event(&hex_encoded_event).expect("Must be Ok");
	println!("Account: {}, Hash: {}", event.who, event.data_hash);

	Ok(())
}
