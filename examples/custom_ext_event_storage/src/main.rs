use avail::staking::types::ValidatorPrefs;
use avail_rust::prelude::*;

// Custom Event
// Implementing HasHeader, codec::Decode and coded::Encode is
// enough to create a custom event
#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomEvent {
	pub who: AccountId,
	pub data_hash: H256,
}
impl HasHeader for CustomEvent {
	const HEADER_INDEX: (u8, u8) = (29, 1);
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Custom Event
	let tx_events = block_tx.events(client.clone()).await?;
	let event = tx_events.first::<CustomEvent>().expect("Should be there");
	println!("Who: {}, Data Hash: {:?}", event.who, event.data_hash);
	/*
		Who: 5EZZm8AKzZw8ti9PSmTZdXCgNEeaE3vs5sNxqkQ6u5NhG8kT, Data Hash: 0x94504118a0ce3537d082e821413a172d745b3059dd9c385eceef64933e81aa72
	*/

	Ok(())
}
