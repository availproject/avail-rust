use avail::utility::events as UtilityEvents;
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
	let events = receipt.tx_events().await?;
	for event in events {
		println!("Pallet Index: {}, Variant index: {}", event.emitted_index.0, event.emitted_index.1,);
		let encoded_event = const_hex::decode(event.encoded.expect("Must be there")).expect("Must be ok");
		if let Some(_e) = UtilityEvents::BatchInterrupted::decode_event(&encoded_event) {
			println!("Found Utility::BatchInterrupted");
		}
		if let Some(_e) = UtilityEvents::BatchCompleted::decode_event(&encoded_event) {
			println!("Found Utility::BatchCompleted");
		}
		if let Some(_e) = UtilityEvents::BatchCompletedWithErrors::decode_event(&encoded_event) {
			println!("Found Utility::BatchCompletedWithErrors");
		}
		if let Some(_e) = UtilityEvents::ItemCompleted::decode_event(&encoded_event) {
			println!("Found Utility::ItemCompleted");
		}
		if let Some(_e) = UtilityEvents::ItemFailed::decode_event(&encoded_event) {
			println!("Found Utility::ItemFailed");
		}
		if let Some(_e) = UtilityEvents::DispatchedAs::decode_event(&encoded_event) {
			println!("Found Utility::DispatchedAs");
		}
	}

	Ok(())
}
