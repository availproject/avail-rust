use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;
	let block = client.block(2042868);

	let timestamp = block.timestamp().await?;
	println!("Timestamp: {}", timestamp);

	let info = block.info().await?;
	println!("Block {}", info);

	let header = block.header().await?;
	println!(
		"Block Height: {}, Block Hash: {:?}, Parent Block Hash: {:?}, State Root: {:?}, Extrinsics Root: {:?}",
		header.number,
		header.hash(),
		header.parent_hash,
		header.state_root,
		header.extrinsics_root
	);

	let author = block.author().await?;
	println!("Block Author: {}", author);

	dbg!(block.event_count().await?);
	dbg!(block.extrinsic_count().await?);
	dbg!(block.weight().await?);
	dbg!(block.extrinsic_weight().await?);
	Ok(())
}
