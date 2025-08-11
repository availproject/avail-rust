//! This example showcases the following actions:
//! - How to subscribe to new headers, blocks and grandpa justifications
//!

use std::time::Duration;

use avail_rust_client::{
	prelude::*,
	subscription::{
		BlockSubscription, GrandpaJustificationJsonSubscription, GrandpaJustificationSubscription, HeaderSubscription,
		SubscriptionBuilder,
	},
};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(TURING_ENDPOINT).await?;

	showcase_header_subscription(&client).await?;
	showcase_block_subscription(&client).await?;
	showcase_best_block_subscription(&client).await?;
	showcase_historical_subscription(&client).await?;
	showcase_grandpa_justification_subscription(&client).await?;
	showcase_grandpa_justification_json_subscription(&client).await?;

	Ok(())
}

async fn showcase_header_subscription(client: &Client) -> Result<(), ClientError> {
	let finalized_height = client.finalized_block_height().await?;
	let sub = SubscriptionBuilder::new().build(client).await?;
	let mut sub = HeaderSubscription::new(client.clone(), sub);

	println!("Fetching new finalized block header...");
	let mut expected_height = finalized_height;
	let header = sub.next().await.ok().flatten().expect("Must be there");
	assert_eq!(header.number, expected_height);

	println!("Fetching new finalized block header...");
	expected_height += 1;
	let header = sub.next().await.ok().flatten().expect("Must be there");
	assert_eq!(header.number, expected_height);

	Ok(())
}

async fn showcase_block_subscription(client: &Client) -> Result<(), ClientError> {
	let finalized_height = client.finalized_block_height().await?;
	let sub = SubscriptionBuilder::new().build(client).await?;
	let mut sub = BlockSubscription::new(client.clone(), sub);

	println!("Fetching new finalized block...");
	let mut expected_height = finalized_height;
	let block = sub.next().await.ok().flatten().expect("Must be there");
	let header = &block.block.header;
	assert_eq!(header.number, expected_height);

	println!("Fetching new finalized block...");
	expected_height += 1;
	let block = sub.next().await.ok().flatten().expect("Must be there");
	let header = &block.block.header;
	assert_eq!(header.number, expected_height);

	Ok(())
}

async fn showcase_historical_subscription(client: &Client) -> Result<(), ClientError> {
	let historical_height = client.finalized_block_height().await?.saturating_sub(100);
	let sub = SubscriptionBuilder::new()
		.block_height(historical_height)
		.build(client)
		.await?;
	let mut sub = BlockSubscription::new(client.clone(), sub);

	println!("Fetching old finalized block...");
	let mut expected_height = historical_height;
	let block = sub.next().await.ok().flatten().expect("Must be there");
	let header = &block.block.header;
	assert_eq!(header.number, expected_height);

	println!("Fetching old finalized block...");
	expected_height += 1;
	let block = sub.next().await.ok().flatten().expect("Must be there");
	let header = &block.block.header;
	assert_eq!(header.number, expected_height);

	Ok(())
}

async fn showcase_best_block_subscription(client: &Client) -> Result<(), ClientError> {
	let best_block_height = client.best_block_height().await?;
	let sub = SubscriptionBuilder::new().follow_best_blocks().build(client).await?;
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
	assert_eq!(header.number, expected_height);

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

async fn showcase_grandpa_justification_subscription(client: &Client) -> Result<(), ClientError> {
	let mut sub = GrandpaJustificationSubscription::new(client.clone(), Duration::from_secs(1), 2100000);

	let justification = sub.next().await?;
	println!(
		"Found grandpa justification at block: {}. Round: {}",
		justification.commit.target_number, justification.round
	);

	let justification = sub.next().await?;
	println!(
		"Found grandpa justification at block: {}. Round: {}",
		justification.commit.target_number, justification.round
	);

	Ok(())
}

async fn showcase_grandpa_justification_json_subscription(client: &Client) -> Result<(), ClientError> {
	let mut sub = GrandpaJustificationJsonSubscription::new(client.clone(), Duration::from_secs(1), 2100223);

	let justification = sub.next().await?;
	println!(
		"Found grandpa justification json at block: {}. Round: {}",
		justification.commit.target_number, justification.round
	);

	let justification = sub.next().await?;
	println!(
		"Found grandpa justification json at block: {}. Round: {}",
		justification.commit.target_number, justification.round
	);

	Ok(())
}
