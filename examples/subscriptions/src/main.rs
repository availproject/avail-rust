//! This example showcases the following actions:
//! - How to subscribe to new headers, blocks and grandpa justifications
//!

use avail_rust_client::{
	prelude::*,
	subscription::{BlockSubscription, GrandpaJustificationsSubscription, HeaderSubscription, Subscriber},
};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	showcase_header_subscription(&client).await?;
	showcase_block_subscription(&client).await?;
	showcase_best_block_subscription(&client).await?;
	showcase_historical_subscription(&client).await?;
	showcase_grandpa_justifications_subscription(&client).await?;

	Ok(())
}

async fn showcase_header_subscription(client: &Client) -> Result<(), ClientError> {
	let finalized_height = client.finalized_block_height().await?;
	let sub = Subscriber::new_finalized_block(5000, finalized_height);
	let mut sub = HeaderSubscription::new(client.clone(), sub);

	println!("Fetching new finalized block header...");
	let mut expected_height = finalized_height;
	let header = sub.next().await.ok().flatten().expect("Must be there");
	println!(
		"Fetched finalized block header with block number: {}, expected block number: {}",
		header.number, expected_height
	);
	if header.number != expected_height {
		return Err("We got the wrong header".into());
	}

	println!("Fetching new finalized block header...");
	expected_height += 1;
	let header = sub.next().await.ok().flatten().expect("Must be there");
	println!(
		"Fetched finalized block header with block number: {}, expected block number: {}",
		header.number, expected_height
	);
	if header.number != expected_height {
		return Err("We got the wrong header".into());
	}

	Ok(())
}

async fn showcase_block_subscription(client: &Client) -> Result<(), ClientError> {
	let finalized_height = client.finalized_block_height().await?;
	let sub = Subscriber::new_finalized_block(5000, finalized_height);
	let mut sub = BlockSubscription::new(client.clone(), sub);

	println!("Fetching new finalized block...");
	let mut expected_height = finalized_height;
	let block = sub.next().await.ok().flatten().expect("Must be there");
	let header = &block.block.header;
	println!(
		"Fetched finalized block with block number: {}, expected block number: {}",
		header.number, expected_height
	);
	if header.number != expected_height {
		return Err("We got the wrong block".into());
	}

	println!("Fetching new finalized block...");
	expected_height += 1;
	let block = sub.next().await.ok().flatten().expect("Must be there");
	let header = &block.block.header;
	println!(
		"Fetched finalized block with block number: {}, expected block number: {}",
		header.number, expected_height
	);
	if header.number != expected_height {
		return Err("We got the wrong block".into());
	}

	Ok(())
}

async fn showcase_historical_subscription(client: &Client) -> Result<(), ClientError> {
	let historical_height = client.finalized_block_height().await?.saturating_sub(100);
	let sub = Subscriber::new_finalized_block(5000, historical_height);
	let mut sub = BlockSubscription::new(client.clone(), sub);

	println!("Fetching old finalized block...");
	let mut expected_height = historical_height;
	let block = sub.next().await.ok().flatten().expect("Must be there");
	let header = &block.block.header;
	println!(
		"Fetched old finalized block with block number: {}, expected block number: {}",
		header.number, expected_height
	);
	if header.number != expected_height {
		return Err("We got the wrong block".into());
	}

	println!("Fetching old finalized block...");
	expected_height += 1;
	let block = sub.next().await.ok().flatten().expect("Must be there");
	let header = &block.block.header;
	println!(
		"Fetched old finalized block with block number: {}, expected block number: {}",
		header.number, expected_height
	);
	if header.number != expected_height {
		return Err("We got the wrong block".into());
	}

	Ok(())
}

async fn showcase_best_block_subscription(client: &Client) -> Result<(), ClientError> {
	let best_block_height = client.best_block_height().await?;
	let sub = Subscriber::new_best_block(1000, best_block_height);
	let mut sub = BlockSubscription::new(client.clone(), sub);

	println!("Fetching new best block...");
	let mut expected_height = best_block_height;
	let block = sub.next().await.ok().flatten().expect("Must be there");
	let header = &block.block.header;
	println!(
		"Fetched best block with block number: {}, block hash: {:?}, expected block number: {}",
		header.number,
		header.hash(),
		expected_height
	);
	if header.number != expected_height {
		return Err("We got the wrong block".into());
	}

	expected_height = best_block_height + 1;

	loop {
		println!("Fetching new best block...");
		let block = sub.next().await.ok().flatten().expect("Must be there");
		let header = &block.block.header;
		println!("Fetched best block with block number: {}, block hash: {:?}", header.number, header.hash(),);

		if header.number == (expected_height - 1) {
			println!(
				"Found fork. Received best block with the same height but different hash. Height: {}, Hash: {:?}",
				header.number,
				header.hash()
			);
		}
		if header.number == expected_height {
			break;
		}
	}

	Ok(())
}

async fn showcase_grandpa_justifications_subscription(client: &Client) -> Result<(), ClientError> {
	let mut sub = GrandpaJustificationsSubscription::new(client.clone(), 1000, 0);

	let (justification, block_height) = sub.next().await?;
	println!("Found grandpa justification at block: {}. Round: {}", block_height, justification.round);

	let (justification, block_height) = sub.next().await?;
	println!("Found grandpa justification at block: {}. Round: {}", block_height, justification.round);

	Ok(())
}
