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
	let events = receipt.tx_events().await?;
	let event = events.find::<CustomEvent>().expect("Must be there");
	println!("Account: {}, Hash: {}", event.who, event.data_hash);

	Ok(())
}
