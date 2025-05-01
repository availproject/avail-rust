use std::time::Duration;

use avail_rust::{prelude::*, transaction::block_state};
use tokio::time::sleep;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = account::alice();

	// Data Submission
	let data = String::from("My Data").into_bytes();
	let options = Options::new().app_id(2);
	let tx = sdk.tx.data_availability.submit_data(data);

	// SubmittedTransaction -> Transaction Hash, and Transaction extra
	let st: SubmittedTransaction = tx.execute(&account, options).await?;
	// At this point it is guaranteed that the transaction was successfully submitted.
	// This does not mean that the transaction will be included in any block because:
	// a) congestion could force the transaction to be dropped
	// b) the transaction could be dropped because we replaced it
	// c) it was so far behind in the queue that it never got the chance to be executed so it got dropped (mortality)
	'outer: loop {
		// TransactionReceipt -> Block height, Block hash, Transaction hash, Transaction index, and Transaction extra.
		// If None it means that the transaction was dropped. This is guaranteed***(pruning could mess this up).
		// This call is extremely cheap and can be done as many times as needed.
		let receipt: TransactionReceipt = st
			.receipt(ReceiptMethod::Default { use_best_block: true })
			.await?
			.unwrap();

		// At this point it is guaranteed that the transaction was observed in a block.
		// If the setting was to wait for finalization then we are done. If the setting was wait for
		// inclusion then the transaction can still be dropped because:
		// a) forks
		// b) forks
		// c) forks

		// !! By default `st.receipt` waits for finalization so the next block of code is only relevant if we instead waited for
		// inclusion. !!
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

/* use avail_rust::prelude::*;
use std::time::SystemTime;

type DataSubmissionCall = avail::data_availability::calls::types::SubmitData;
type ApplicationKeyCreatedEvent = avail::data_availability::events::ApplicationKeyCreated;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = account::alice();

	// Application Key Creation
	let time = std::format!("{:?}", SystemTime::now());
	let key = time.into_bytes();
	let tx = sdk.tx.data_availability.create_application_key(key);
	let res = tx.execute_and_watch(&account, Options::default()).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions must be successful");

	let events = res.events.as_ref().unwrap();
	let event = events.find_first::<ApplicationKeyCreatedEvent>().unwrap();
	let Some(event) = event else {
		return Err("Failed to get Application Key Created Event".into());
	};
	let app_id = event.id.0;

	// Data Submission
	let data = String::from("My Data").into_bytes();
	let options = Options::new().app_id(app_id);
	let tx = sdk.tx.data_availability.submit_data(data);
	let res = tx.execute_and_watch(&account, options).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions must be successful");

	println!(
		"Block Hash: {:?}, Block Number: {}, Tx Hash: {:?}, Tx Index: {}",
		res.block_hash, res.block_number, res.tx_hash, res.tx_index
	);

	// Events
	let events = res.events.as_ref().unwrap();
	for event in events.iter() {
		let tx_index = match event.phase() {
			subxt::events::Phase::ApplyExtrinsic(x) => Some(x),
			_ => None,
		};

		println!(
			"Pallet Name: {}, Pallet Index: {}, Event Name: {}, Event Index: {}, Event Position: {}, Tx Index: {:?}",
			event.pallet_name(),
			event.pallet_index(),
			event.variant_name(),
			event.variant_index(),
			event.index(),
			tx_index,
		);
	}

	// Decoding
	let decoded = res.decode_as::<DataSubmissionCall>().await?;
	let Some(decoded) = decoded else {
		return Err("Failed to get Data Submission Call data".into());
	};

	let data = to_ascii(decoded.data.0).unwrap();
	println!("Call data: {:?}", data);

	println!("Data Submission finished correctly");

	Ok(())
}
 */
