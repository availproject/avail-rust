use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Block State
	let chain_info = client.chain().chain_info().await?;
	let best_block_state = client.chain().block_state(chain_info.best_height).await?;
	let finalized_block_state = client.chain().block_state(chain_info.finalized_height).await?;
	let historical_block_state = client.chain().block_state(2000000).await?;
	let non_existing_block_state = client.chain().block_state(100000000).await?;
	println!("Best Block State:                     {}", best_block_state);
	println!("Finalized Block State:                {}", finalized_block_state);
	println!("Historical (2000000) Block State:     {}", historical_block_state);
	println!("Non Existing (100000000) Block State: {}", non_existing_block_state);

	Ok(())
}

/*
	Expected Output:

	Best Block State:                     Included
	Finalized Block State:                Finalized
	Historical (2000000) Block State:     Finalized
	Non Existing (100000000) Block State: DoesNotExist
*/
