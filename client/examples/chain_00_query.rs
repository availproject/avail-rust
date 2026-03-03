use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::connect(LOCAL_ENDPOINT).await?;

	// Chain interface
	//
	// Chain interface is the backbone of all other interfaces that are build on top of it.
	// Block, Subscription, Submission, etc. all of the use .chain() underneath them.
	// Chain has around 50 methods and documenting all of them would be a fools errant.
	//
	// Instead, I advise you to check them yourself and see if any of the methods fits your bill.
	// Here I will just show the ones that are needed and used the most.
	//
	// .info() 			- returns the genesis hash together with best and finalized block hashes and heights.
	// .block_hash() 	- returns block hash.
	// .block_height()	- returns block height.
	let chain = client.chain();
	let _ = chain.info().await?;
	let _ = chain.block_hash(Some(1)).await?;
	let _ = chain.block_height(H256::zero()).await?;

	// To quickly query only best or finalized block information, .best() and .finalized()
	// interfaces can be used. They are build on top of .chain() and are here just to
	// provide a shortcut. They only offer a limited set of information to be queried so
	// more than often you will rely on .chain()
	let best = client.best();
	let _ = best.block_hash().await?;
	let _ = best.block_height().await?;
	let _ = best.block_info().await?;

	let finalized = client.finalized();
	let _ = finalized.block_hash().await?;
	let _ = finalized.block_height().await?;
	let _ = finalized.block_info().await?;

	Ok(())
}
