//! This example showcases the following actions:
//! - Fetching Block Header
//! - Fetching Block State
//! - Fetching Block Hash from Block Height
//! - Fetching Block Height from Block Hash
//! - Fetching Block Hash and Height
//!
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	let client = Client::new(TURING_ENDPOINT).await?;

	block_header_example(&client).await?;
	block_state_example(&client).await?;
	block_height_example(&client).await?;
	block_hash_example(&client).await?;

	Ok(())
}

pub async fn block_header_example(client: &Client) -> Result<(), ClientError> {
	// Custom Block Header
	let block_hash = H256::from_str("0x149d4a65196867e6693c5bc731a430ebb4566a873f278d712c8e6d36aec7cb78")?;
	let block_header = client.block_header(block_hash).await?.expect("Must be there");
	assert_eq!(block_header.number, 100);

	// Custom Block Header (On Failure and None it retries)
	let block_header = client
		.block_header_with_retries(block_hash)
		.await?
		.expect("Must be there");
	assert_eq!(block_header.number, 100);

	// Best Block Header
	let block_header = client.best_block_header().await?;
	assert!(block_header.number > 100);

	// Finalized Block Header
	let block_header = client.finalized_block_header().await?;
	assert!(block_header.number > 100);

	Ok(())
}

pub async fn block_state_example(client: &Client) -> Result<(), ClientError> {
	// Custom Block State (unknown block hash)
	let loc = BlockLocation::from((H256::default(), 100u32));
	let block_state = client.block_state(loc).await?;
	assert_eq!(block_state, BlockState::Discarded);

	// Custom Block State (unknown block height)
	let loc = BlockLocation::from((H256::default(), 50_000_000u32));
	let block_state = client.block_state(loc).await?;
	assert_eq!(block_state, BlockState::DoesNotExist);

	// Best Block State
	let loc = client.best_block_loc().await?;
	let block_state = client.block_state(loc).await?;
	assert_eq!(block_state, BlockState::Included);

	// Finalized Block State
	let loc = client.finalized_block_loc().await?;
	let block_state = client.block_state(loc).await?;
	assert_eq!(block_state, BlockState::Finalized);

	Ok(())
}

pub async fn block_height_example(client: &Client) -> Result<(), ClientError> {
	// Block Hash to Block Height
	let block_hash = H256::from_str("0x149d4a65196867e6693c5bc731a430ebb4566a873f278d712c8e6d36aec7cb78")?;
	let block_height = client.block_height(block_hash).await?.expect("Must be there");
	assert_eq!(block_height, 100);

	// Block Hash to Block Height (On Failure and None it retries)
	let block_height = client
		.block_height_with_retries(block_hash)
		.await?
		.expect("Must be there");
	assert_eq!(block_height, 100);

	// Best Block Height
	let best_height = client.best_block_height().await?;
	assert!(best_height > 100);

	// Finalized Block Height
	let finalized_height = client.finalized_block_height().await?;
	assert!(best_height > finalized_height);

	Ok(())
}

pub async fn block_hash_example(client: &Client) -> Result<(), ClientError> {
	// Block Height to Block Hash
	let block_hash = client.block_hash(100).await?.expect("Must be there");
	assert_eq!(
		std::format!("{:?}", block_hash),
		"0x149d4a65196867e6693c5bc731a430ebb4566a873f278d712c8e6d36aec7cb78"
	);

	// Block Height to Block Hash (On Failure and None it retries)
	let block_hash = client.block_hash_with_retries(100).await?.expect("Must be there");
	assert_eq!(
		std::format!("{:?}", block_hash),
		"0x149d4a65196867e6693c5bc731a430ebb4566a873f278d712c8e6d36aec7cb78"
	);

	// Best Block Hash
	let best_hash = client.best_block_hash().await?;

	// Finalized Block Hash
	let finalized_hash = client.finalized_block_hash().await?;
	assert_ne!(best_hash, finalized_hash);

	Ok(())
}
