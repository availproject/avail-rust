use avail_rust_client::prelude::*;
use chrono::{Datelike, TimeDelta, Timelike};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;
	let block = client.block(2042867);

	// Fetching Block Timestamp
	let timestamp = block.timestamp().await?;

	// Converting Block Timestamp to DateTime (Adding two hours to match my timezone)
	let date_time = chrono::DateTime::from_timestamp_millis(timestamp as i64).expect("Conversion should work");
	let date_time_local = date_time
		.checked_add_signed(TimeDelta::hours(2))
		.expect("Delta should work");

	let date_time = std::format!(
		"{}/{}/{} {}:{}:{}",
		date_time_local.day(),
		date_time_local.month(),
		date_time_local.year(),
		date_time_local.hour(),
		date_time_local.minute(),
		date_time_local.second()
	);
	println!("1. Timestamp: {}, Date Time: {}", timestamp, date_time);

	// Event Count & Extrinsic Count
	let event_count = block.event_count().await?;
	let extrinsic_count = block.extrinsic_count().await?;
	println!("2. Event Count: {}, Extrinsic Count: {}", event_count, extrinsic_count);

	// Author, Header and Info
	let author = block.author().await?;
	let header = block.header().await?;
	println!(
		"3. Block Height: {}, Block Author: {},  Block Hash: {:?}, Block Parent Hash: {:?}, Extrinsics Root: {:?}, State Root: {:?}",
		header.number,
		author,
		header.hash(),
		header.parent_hash,
		header.extrinsics_root,
		header.state_root,
	);

	// Simple block height and hash information
	let info = block.info().await?;
	println!("Block Height: {}, Block Hash: {:?}", info.height, info.hash);

	// Extrinsic and Block Weight
	let extrinsic_weight = block.extrinsic_weight().await?;
	let block_weight = block.weight().await?;
	let block_weight =
		block_weight.mandatory.ref_time + block_weight.normal.ref_time + block_weight.operational.ref_time;
	println!("4. Extrinsic Weight: {}, Block Weight: {}", extrinsic_weight.ref_time, block_weight);

	// Logs (Digest)
	let header = block.header().await?;
	println!("5. Logs (Digest) Count: {}", header.digest.logs.len());

	Ok(())
}

/*
	Expected Output:

	1. Timestamp: 1760954220001, Date Time: 20/10/2025 11:57:0
	2. Event Count: 3, Extrinsic Count: 2
	3. Block Height: 2042867, Block Author: 5HeP6FZoHcDJxGgF4TauP4yyZGfDTzZtGB28RHvxXjRSm6h6,  Block Hash: 0x45c4fb5b83053dc5816eb0d532eba7dbd971921946dd56031937542291de5a7d, Block Parent Hash: 0x625b3e9d563d73a4a639ca82ccbe4e2c97c931ff339f5148ea31ea66fe1ec576, Extrinsics Root: 0x0eb97eb36ef9f9a265c633682c0c10c2859719b55edb41d6c782bfb3c1be7dde, State Root: 0x336c40c0ca6f175570d1c489512d3f4fc5a1e5be9fd3fe565009e2a4c8da5c90
	Block Height: 2042867, Block Hash: 0x45c4fb5b83053dc5816eb0d532eba7dbd971921946dd56031937542291de5a7d
	4. Extrinsic Weight: 25047612000, Block Weight: 28104773000
	5. Logs (Digest) Count: 2
*/
