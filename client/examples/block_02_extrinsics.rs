use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::connect(LOCAL_ENDPOINT).await?;

	// Fetching extrinsics
	//
	// The easiest way to fetch extrinsics from a particular block is via block.extrinsics().all() interface.
	// This will fetch all extrinsics executed in that block.
	//
	// We can adjust what extrinsics we get back by setting the first param to:
	// - None
	// - Some(list) - [!!Up to 30 items!!]
	// Where each elements of the list can be
	// - AllowedExtrinsic::TxHash 		- hash of extrinsic
	// - AllowedExtrinsic::TxIndex 		- index of extrinsic
	// - AllowedExtrinsic::Pallet 		- pallet id of extrinsic
	// - AllowedExtrinsic::PalletCall	- call id (pallet id + variant id) of extrinsic
	// If None then all extrinsic are allowed and fetched. If Some and the list is empty then
	// not a single extrinsic will be allowed and fetched. So the list acts as a whitelist
	// construct and you whitelist any combination of hashes, indices, pallets and/or calls.
	//
	// The second param allows us to filter by signature. By default no signature filtering is done.
	// This is useful if we want to fetch extrinsic that are submitted by an specific account.
	let block = client.block(1);
	let _ = block.extrinsics().all(None, Default::default()).await?;

	// If we already know what extrinsic we want, we can use .all_as() and it will fetch extrinsics only
	// related to that specific call type. The only drawback is that it can only fetch one specific
	// type unless you manually implement a struct that can handle multiple types...
	let _ = block
		.extrinsics()
		.all_as::<avail::timestamp::tx::Set>(Default::default())
		.await?;

	// There are some QoL interfaces that is build on top of this:
	//
	// .get() 		- returns only one extrinsic from the block defined by either hash, index or string.
	// .get_as() 	- same as .get() but with the call type already defined.
	// .first() 	- returns the first extrinsic from the block that matches the filters.
	// .first_as()	- same as .first() but with the call type already defined.
	// .last() 		- returns the last extrinsic from the block that matches the filters.
	// .last_as() 	- same as .last() but with the call type already defined.
	// .count() 	- returns the count of extrinsics in a block
	// .exists() 	- returns true if an extrinsic exsits in the block with given filters
	let _ = block.extrinsics().get(0).await?;
	let _ = block.extrinsics().first(None, Default::default()).await?;
	let _ = block.extrinsics().last(None, Default::default()).await?;
	let _ = block.extrinsics().count(None, Default::default()).await?;
	let _ = block.extrinsics().exists(None, Default::default()).await?;

	// The .rpc() method gives you one more level of freedom compared to .all() method.
	// You can define the amount of data to fetch from the RPC:
	// - DataFormat::None 		- nothing will be fetched
	// - DataFormat::Call 		- only the extrinsic call will be fetched
	// - DataFormat::Extrinsic	- The whole extrinsic will be fetched
	let _ = block
		.extrinsics()
		.rpc(None, Default::default(), Default::default())
		.await?;

	Ok(())
}
