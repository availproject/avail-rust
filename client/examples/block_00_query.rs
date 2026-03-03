use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::connect(LOCAL_ENDPOINT).await?;

	// Fetching block relevant information
	//
	// Sometimes we don't need extrinsics or events from a block and instead
	// we need the header, the author, or the total weight. All that together
	// with some other information can be retrieved from the block interface.
	//
	// .author() 			- returns the author of the block. [!!There is no author for block height 0!!]
	// .header() 			- returns the header of the block.
	// .info() 				- returns the block height and block hash.
	// .justification()		- returns the grandpa justification if exists.
	// .timestamp() 		- returns the timestamp of the block. [!!The timestamp is 0 for block height 0!!]
	// .weight() 			- returns the weight of the block.
	// .nonce() 			- returns the nonce of an account for that specific block.
	// .metadata() 			- returns the metadata of the block.
	// .extrinsic_count()	- returns the number of extrinsics there is in the block
	// .event_count()		- returns the number of events there is in the block
	let block = client.block(1);
	let author = block.author().await?;
	let header = block.header().await?;
	let info = block.info().await?;
	let justification = block.justification().await?;
	let ts = block.timestamp().await?;
	let weight = block.weight().await?;
	let nonce = block.nonce(alice().public_key().to_account_id()).await?;
	let _ = block.metadata().await?;
	let ext_count = block.extrinsic_count().await?;
	let ev_count = block.event_count().await?;

	println!("Author: {}, Block Height: {}, Block Hash: {:?}", author, info.height, info.hash);
	println!("Has Justification: {}, Timestamp: {}", justification.is_some(), ts);
	println!("Weight: {}, Nonce: {}, ", weight.total_weight(), nonce);
	println!("Digest Logs Count: {}", header.digest.logs.len());
	println!("Extrinsic Count: {}, Event Count: {}", ext_count, ev_count);
	/*
		Author: 5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY, Block Height: 1, Block Hash: 0x725c356c26ff07765df64b7e35bc20346fa65d151f7e1b33f0225107d804ce34
		Has Justification: true, Timestamp: 1772405514000
		Weight: 42666390162, Nonce: 0,
		Digest Logs Count: 3
		Extrinsic Count: 3, Event Count: 3
	*/

	Ok(())
}
