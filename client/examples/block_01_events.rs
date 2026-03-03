use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::connect(LOCAL_ENDPOINT).await?;

	// Fetching block events
	//
	// The easiest way to fetch events from a particular block is via block.events().all() interface.
	// This will fetch all events emitted in that block which includes extrinsic and non-extrinsic events.
	//
	// We can adjust what events we get back by setting the input param to:
	// - AllowedEvents::All 			  	- fetches all events, (it's the default)
	// - AllowedEvents::OnlyExtrinsics    	- fetches only extrinsic events
	// - AllowedEvents::OnlyNonExtrinsics 	- fetches non-extrinsic events
	// - AllowedEvents::Only(Vec<u32>) 		- fetches events for specific extrinsic indices.
	let block = client.block(1);
	let _ = block.events().all(Default::default()).await?;

	// There are some QoL interfaces that is build on top of this:
	//
	// .system() 			- returns only system events
	// .extrinsic() 		- returns only events for one specific extrinsic index
	// .extrinsic_weight() 	- returns the weight of all extrinsic combined
	let _ = block.events().system().await?;
	let _ = block.events().extrinsic(0).await?;
	let _ = block.events().extrinsic_weight().await?;

	// If you just need the event count you can call .event_count()
	let _ = block.events().event_count();

	// .all() interface builds on top of existing RPC. If you need raw values from the underlying RCP
	// you can call .rpc(). The first parameter is the same as for .all() while the second allows you
	// to define if you want or don't want events data to be fetched.
	let _ = block.events().rpc(Default::default(), true).await?;

	Ok(())
}
