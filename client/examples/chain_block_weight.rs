use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Block Weight
	let block_weight = client.chain().block_weight(2000000).await?;
	println!(
		"Block Mandatory Weight: {}, Block Normal Weight: {}, Block Operational Weight: {}",
		block_weight.mandatory.ref_time, block_weight.normal.ref_time, block_weight.operational.ref_time
	);

	Ok(())
}

/*
	Expected Output:

	Block Mandatory Weight: 27854773000, Block Normal Weight: 360671400000, Block Operational Weight: 0
*/
