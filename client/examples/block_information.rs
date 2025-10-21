use avail_rust_client::prelude::*;
use chrono::{Datelike, TimeDelta, Timelike};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;
	let block = client.block(2042867);

	// Timestamp
	// Adding two hours to match my timezone. Change this so it matches your timezone.
	let timestamp = block.timestamp().await?;
	// Formatting
	let date_time = chrono::DateTime::from_timestamp_millis(timestamp as i64).expect("Conversion should work");

	// Adding two hours to match my timezone.
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

	let header = block.header().await?;
	println!("5. Logs (Digest) Count: {}", header.digest.logs.len());

	Ok(())
}
