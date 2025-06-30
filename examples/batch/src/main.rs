use avail::utility::events::Event as UtilityEvent;
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let balances = client.tx().balances();
	let c1 = balances
		.transfer_keep_alive(bob().account_id(), constants::ONE_AVAIL)
		.call;
	let c2 = balances
		.transfer_keep_alive(dave().account_id(), constants::ONE_AVAIL)
		.call;

	let tx = client.tx().utility().batch_all(vec![c1, c2]);
	let st = tx.sign_and_submit(&alice(), Options::new(None)).await?;
	let Some(receipt) = st.receipt(false).await? else {
		return Err("Transaction got dropped. This should never happen in a local network.".into());
	};

	let block_state = receipt.block_state().await?;
	println!("Block State: {:?}", block_state);

	// Fetching and displaying Transaction Events
	let event_group = receipt.tx_events().await?;
	for event in event_group.events {
		println!(
			"Pallet Index: {}, Variant index: {}",
			event.emitted_index.0, event.emitted_index.1,
		);
		let encoded_event = hex::decode(event.encoded.expect("Must be there")).expect("Must be ok");
		let Ok(event) = RuntimeEvent::try_from(&encoded_event) else {
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
