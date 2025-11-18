use avail_rust_client::prelude::*;

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

	let block = client.block(2042845);
	let extrinsic_events = block.events().extrinsic(1).await?;
	let event = extrinsic_events.first::<CustomEvent>().expect("Should be there");
	println!("Who: {}, Data Hash: {:?}", event.who, event.data_hash);
	/*
		Who: 5CFyESc2tw4WX4uZFhACrZTg8JP1NRdQGaQVUaWjjdLJy1dq, Data Hash: 0x69c2ae9d6d95c7b52588b189a1395bd63f30cb2144d590db8fcd42ccf03c8744
	*/

	Ok(())
}
