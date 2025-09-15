use avail::utility::{events as UtilityEvents, tx::BatchAll as UtilityBatchAll};
use avail_rust_client::{avail::RuntimeCall, error::Error, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Error> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let balances = client.tx().balances();
	let c1 = balances.transfer_keep_alive(bob().account_id(), constants::ONE_AVAIL);
	let c2 = balances.transfer_keep_alive(dave().account_id(), constants::ONE_AVAIL);

	let tx = client.tx().utility().batch_all(vec![c1, c2]);
	let st = tx.sign_and_submit(&alice(), Options::default()).await?;
	let Some(receipt) = st.receipt(false).await? else {
		return Err("Transaction got dropped. This should never happen in a local network.".into());
	};

	let block_state = receipt.block_state().await?;
	println!("Block State: {:?}", block_state);

	// Fetching and displaying Transaction Events
	let events = receipt.events().await?;
	if events.is_present::<UtilityEvents::BatchInterrupted>() {
		println!("Found Utility::BatchInterrupted");
	}
	if events.is_present::<UtilityEvents::BatchCompleted>() {
		println!("Found Utility::BatchCompleted");
	}
	if events.is_present::<UtilityEvents::BatchCompletedWithErrors>() {
		println!("Found Utility::BatchCompletedWithErrors");
	}
	if events.is_present::<UtilityEvents::ItemCompleted>() {
		println!("Found Utility::ItemCompleted");
	}
	if events.is_present::<UtilityEvents::ItemFailed>() {
		println!("Found Utility::ItemFailed");
	}
	if events.is_present::<UtilityEvents::DispatchedAs>() {
		println!("Found Utility::DispatchedAs");
	}

	// Decoding batch call
	let block = client.block(receipt.block_ref);
	let tx: block::BlockSignedExtrinsic<UtilityBatchAll> = block
		.tx
		.get::<UtilityBatchAll>(receipt.tx_ref)
		.await?
		.expect("Should be there");

	// Not all calls are decodable.
	let decoded_calls = tx.call.decode_calls()?;
	for call in decoded_calls {
		let RuntimeCall::BalancesTransferKeepAlive(tx) = call else {
			return Err("Expected Balance Transfer Keep Alive".into());
		};

		println!("Dest: {:?}, Amount: {}", tx.dest, tx.value);
	}

	Ok(())
}
