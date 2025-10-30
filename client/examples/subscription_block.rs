use avail_rust_client::{prelude::*, subscription::BlockSub};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// By default it subscribes to finalized block
	let mut sub = BlockSub::new(client.clone());
	let (next_block, next_info) = sub.next().await?;
	let (prev_block, prev_info) = sub.prev().await?;
	println!(
		"Finalized Next:      Block Height: {}, Block Author: {}",
		next_info.height,
		next_block.author().await?
	);
	println!(
		"Finalized Previous:  Block Height: {}, Block Author: {}",
		prev_info.height,
		prev_block.author().await?
	);

	// Best Blocks
	let mut sub = BlockSub::new(client.clone());
	sub.use_best_block(true);
	let (next_block, next_info) = sub.next().await?;
	let (prev_block, prev_info) = sub.prev().await?;
	println!(
		"Best Next:           Block Height: {}, Block Author: {}",
		next_info.height,
		next_block.author().await?
	);
	println!(
		"Best Previous:       Block Height: {}, Block Author: {}",
		prev_info.height,
		prev_block.author().await?
	);

	// Historical Blocks
	let mut sub = BlockSub::new(client.clone());
	sub.set_block_height(2000000);
	let (next_block, next_info) = sub.next().await?;
	let (prev_block, prev_info) = sub.prev().await?;
	println!(
		"Historical Next:     Block Height: {}, Block Author: {}",
		next_info.height,
		next_block.author().await?
	);
	println!(
		"Historical Previous: Block Height: {}, Block Author: {}",
		prev_info.height,
		prev_block.author().await?
	);

	Ok(())
}

/*
	Expected Output:

	Finalized Next:      Block Height: 2504096, Block Author: 5CAfDa2yVHCWjXrZ66iCCVm3v9pUzAVguadXbARne1CnxtSb
	Finalized Previous:  Block Height: 2504095, Block Author: 5CAPbuoFSr3p5v3DxWRqbeYMkZMmioWTTrQHvqbBSZE9DoWq
	Best Next:           Block Height: 2504097, Block Author: 5DPzoTT2NsPb2u5YNkvCmrW55Hv49qzczqD8D78scqsWk6x8
	Best Previous:       Block Height: 2504096, Block Author: 5CAfDa2yVHCWjXrZ66iCCVm3v9pUzAVguadXbARne1CnxtSb
	Historical Next:     Block Height: 2000000, Block Author: 5GQjARS9nVu5t3NrBZGhdUKKpwdj1xvnek9rNk7UKaH7DHoJ
	Historical Previous: Block Height: 1999999, Block Author: 5DtTjsDx6NXtni43VFFZkrEqfpyFnZDJ2MhwpgGJWrUejV29
*/
