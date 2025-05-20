use avail_rust::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_logging();
	let client = Client::new(LOCAL_ENDPOINT).await?;
	let signer = alice();

	// Transaction Creation
	let submittable_tx = client.tx().data_availability().submit_data(vec![0, 1, 2, 3, 4, 5]);

	// Transaction Submission
	// SubmittedTransaction -> Transaction Hash, and Transaction extra
	let st = submittable_tx
		.sign_and_submit(&signer, Options::new().app_id(2))
		.await?;
	// At this point it is guaranteed that the transaction was successfully submitted.
	// This does not mean that the transaction will be included in any block because:
	// a) congestion could force the transaction to be dropped
	// b) the transaction could be dropped because we replaced it
	// c) it was so far behind in the queue that it never got the chance to be executed so it got dropped (mortality)

	'outer: loop {
		// TransactionReceipt -> Block height, Block hash, Transaction hash, Transaction index, and Transaction extra.
		// If None it means that the transaction was dropped. This is guaranteed***(pruning could mess this up).
		// This call is extremely cheap and can be done as many times as needed.
		let receipt: TransactionReceipt = st.receipt(true).await?.unwrap();

		// At this point it is guaranteed that the transaction was observed in a block.
		// If the setting was to wait for finalization then we are done. If the setting was wait for
		// best block then the transaction can still be dropped because:
		// a) forks
		// b) forks
		// c) forks

		// !! By default `st.receipt` waits for finalization so the next block of code is only relevant if we instead waited for
		// best block. !!
		loop {
			let block_state: BlockState = receipt.block_state().await?;
			match block_state {
				BlockState::Included => {
					println!("Included.");
					()
				},
				BlockState::Finalized => {
					println!("Finalized.");
					return Ok(());
				},
				// Discarded means that the block that we got from `st.receipt` got discarded.
				// Running `st.receipt` again will give us the correct block height and block hash.
				BlockState::Discarded => {
					println!("Discarded.");
					break 'outer;
				},
				// Due to pruning settings that block does not exist anymore. What exactly needs to be done at this point is
				// still unclear to me.
				BlockState::DoesNotExist => {
					println!("DoesNotExist.");
					unimplemented!();
				},
			};
			sleep(Duration::from_secs(5)).await;
		}
	}

	Ok(())
}
