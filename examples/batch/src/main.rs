use avail::utility::events::Event as UtilityEvent;
use avail_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_logging();
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let balances = client.tx().balances();
	let c1 = balances.transfer_keep_alive(bob().account_id(), ONE_AVAIL).call;
	let c2 = balances.transfer_keep_alive(dave().account_id(), ONE_AVAIL).call;

	let tx = client.tx().utility().batch_all(vec![c1, c2]);
	let st = tx.sign_and_submit(&alice(), Options::new()).await?;
	let Some(receipt) = st.receipt(false).await? else {
		return Err("Transaction got dropped. This should never happen in a local network.".into());
	};

	let block_state = receipt.block_state().await?;
	println!("Block State: {:?}", block_state);

	// Fetching and displaying Transaction Events
	let (tx_index, block_hash) = (receipt.tx_location.index, receipt.block_id.hash);
	let events_client = client.event_client();
	let events = events_client.transaction_events(tx_index, block_hash).await?;
	for event in events {
		println!(
			"Pallet Index: {}, Variant index: {}",
			event.pallet_index(),
			event.variant_index()
		);
		let Ok(event) = RuntimeEvent::try_from(&event) else {
			continue;
		};

		let RuntimeEvent::Utility(ut) = event else { continue };
		match ut {
			UtilityEvent::BatchInterrupted { index: _, error: _ } => println!("Found Utility::BatchInterrupted"),
			UtilityEvent::BatchCompleted => println!("Found Utility::BatchCompleted"),
			UtilityEvent::BatchCompletedWithErrors => println!("Found Utility::BatchCompletedWithErrors"),
			UtilityEvent::ItemCompleted => println!("Found Utility::ItemCompleted"),
			UtilityEvent::ItemFailed { error: _ } => println!("Found Utility::ItemFailed"),
			UtilityEvent::DispatchedAs { result: _ } => println!("Found Utility::DispatchedAs"),
		}
	}

	Ok(())
}
