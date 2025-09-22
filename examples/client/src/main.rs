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
	let block_header = client
		.rpc()
		.block_header(Some(block_hash))
		.await?
		.expect("Must be there");
	assert_eq!(block_header.number, 100);

	// Best Block Header
	let block_header = client.best().block_header().await?;
	assert!(block_header.number > 100);

	// Finalized Block Header
	let block_header = client.finalized().block_header().await?;
	assert!(block_header.number > 100);

	Ok(())
}

pub async fn block_state_example(client: &Client) -> Result<(), ClientError> {
	// Custom Block State (unknown block hash)
	let info = BlockRef::from((H256::default(), 100u32));
	let block_state = client.rpc().block_state(info).await?;
	assert_eq!(block_state, BlockState::Discarded);

	// Custom Block State (unknown block height)
	let info = BlockRef::from((H256::default(), 50_000_000u32));
	let block_state = client.rpc().block_state(info).await?;
	assert_eq!(block_state, BlockState::DoesNotExist);

	// Best Block State
	let info = client.best().block_info().await?;
	let block_state = client.rpc().block_state(info).await?;
	assert_eq!(block_state, BlockState::Included);

	// Finalized Block State
	let info = client.finalized().block_info().await?;
	let block_state = client.rpc().block_state(info).await?;
	assert_eq!(block_state, BlockState::Finalized);

	Ok(())
}

pub async fn block_height_example(client: &Client) -> Result<(), ClientError> {
	// Block Hash to Block Height
	let block_hash = H256::from_str("0x149d4a65196867e6693c5bc731a430ebb4566a873f278d712c8e6d36aec7cb78")?;
	let block_height = client.rpc().block_height(block_hash).await?.expect("Must be there");
	assert_eq!(block_height, 100);

	// Best Block Height
	let best_height = client.best().block_height().await?;
	assert!(best_height > 100);

	// Finalized Block Height
	let finalized_height = client.finalized().block_height().await?;
	assert!(best_height > finalized_height);

	Ok(())
}

pub async fn block_hash_example(client: &Client) -> Result<(), ClientError> {
	// Block Height to Block Hash
	let block_hash = client.rpc().block_hash(Some(100)).await?.expect("Must be there");
	assert_eq!(
		std::format!("{:?}", block_hash),
		"0x149d4a65196867e6693c5bc731a430ebb4566a873f278d712c8e6d36aec7cb78"
	);

	// Best Block Hash
	let best_hash = client.best().block_hash().await?;

	// Finalized Block Hash
	let finalized_hash = client.finalized().block_hash().await?;
	assert_ne!(best_hash, finalized_hash);

	Ok(())
}
