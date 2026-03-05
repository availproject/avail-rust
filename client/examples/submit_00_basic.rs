use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::connect(LOCAL_ENDPOINT).await?;
	let signer = Account::new_from_str("//Bob")?;

	// Submitting a transaction
	//
	// The typical flow is: build a call, submit it, then wait for a receipt. The simplest path
	// is .submit() which signs and sends the transaction in one RPC call. It returns a
	// SubmittedTransaction handle that we can use to track the transaction's progress.
	let tx = client.tx().data_availability().submit_data(2, "Hello Avail!");
	let submitted = tx.submit(&signer, Default::default()).await?;
	println!("Ext Hash: {:?}", submitted.ext_hash);

	// Waiting for inclusion
	//
	// After submission we usually want to wait until the transaction lands in a block. There
	// are three methods on SubmittedTransaction, each giving a different level of detail:
	//
	// .find_receipt(opts)  - returns a FindReceiptOutcome enum (Found / NotFound / TimedOut)
	// .receipt(opts)       - same but returns the receipt directly or an error
	// .outcome(opts)       - returns (TransactionReceipt, BlockEvents) in one shot
	//
	// The opts parameter accepts a BlockQueryMode directly (Finalized or Best) or a WaitOption
	// for finer control over timeout duration.
	let receipt = submitted.receipt(BlockQueryMode::Finalized).await?;
	let timestamp = receipt.timestamp().await?;
	println!("Included: height={}, hash={:?}, timestamp={}", receipt.block_height, receipt.block_hash, timestamp);

	// Pre-submission inspection
	//
	// Before submitting we can estimate fees and inspect weight without actually sending
	// anything to the chain. Both methods accept an optional block hash to pin the estimation
	// to a specific state root — pass None for the latest.
	//
	// .estimate_call_fees(at)                    - fee estimate from the unsigned call
	// .estimate_extrinsic_fees(signer, opts, at) - fee estimate from the fully signed extrinsic
	// .call_info(at)                             - runtime dispatch info including weight
	let tx = client.tx().data_availability().submit_data(2, "Fee check");
	let fee = tx.estimate_call_fees(None).await?;
	println!("Fee: {}", fee.final_fee());

	let weight = tx.call_info(None).await?.weight;
	println!("Weight: {}", weight.ref_time);

	// Convenience: submit and wait in one call
	//
	// If you don't need the intermediate SubmittedTransaction handle, these two methods combine
	// submission and waiting into a single call:
	//
	// .submit_and_wait_for_receipt(signer, opts, wait) - returns TransactionReceipt
	// .submit_and_wait_for_outcome(signer, opts, wait) - returns (TransactionReceipt, BlockEvents)
	//
	// The outcome variant is handy when you want to check events (e.g. ExtrinsicSuccess) right away.
	let tx = client.tx().data_availability().submit_data(2, "Full flow");
	let (receipt, events) = tx
		.submit_and_wait_for_outcome(&signer, Options::new(), BlockQueryMode::Best)
		.await?;
	println!("Height: {}, Success: {}", receipt.block_height, events.is_extrinsic_success_present());

	// Reading back the submitted data
	//
	// Once we have a receipt we can decode the extrinsic that was included in the block. This
	// gives us back the original call with its parameters.
	let ext = receipt.extrinsic::<avail::data_availability::tx::SubmitData>().await?;
	println!("Data: {:?}", String::from_utf8(ext.call.data.to_vec()));

	Ok(())
}
