use crate::ReturnResult;
use avail_rust::block::rpc_block_overview::BlockBuilder;
use avail_rust::client::rpc::rpc_block_overview::{SignatureFilterOptions, TransactionFilterOptions};
/* use avail_rust::block::rpc_block_data::BlockBuilder;
use avail_rust::client::rpc::rpc_block_data::{CallFilter, EventFilter, PhaseFilterOptions, TransactionFilterOptions}; */
use avail_rust::prelude::{dev_accounts::*, *};
use std::time::Duration;
use tokio::time::sleep;

pub async fn run() -> ReturnResult {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let mut tx_filter = TransactionFilterOptions::default();
	tx_filter = TransactionFilterOptions::TxHash(vec![H256::from_str(
		"0xa719094b7be31c79dbf412561f28675f1f3c6294e07b296660f9f74143a0417a",
	)
	.unwrap()]);
	let mut sig_filter = SignatureFilterOptions::default();
	sig_filter.nonce = Some(0);

	let block = BlockBuilder::new(HashIndex::Index(5))
		.transaction_filter(tx_filter)
		.signature_filter(sig_filter)
		.fetch_events(true)
		.enable_event_decoding(true)
		.enable_call_decoding(true)
		.build(&client)
		.await?;
	dbg!(block);

	/* 	let mut call_filter = CallFilter::default();
	   call_filter.transaction = TransactionFilterOptions::TxIndex(vec![1]);
	   let mut event_filter = EventFilter::default();
	   event_filter.phase = PhaseFilterOptions::TxIndex(vec![1]);
	   let builder = BlockBuilder::new(HashIndex::Index(9));
	   let builder = builder
		   .fetch_events(true)
		   .fetch_calls(true)
		   .call_filter(call_filter)
		   .event_filter(event_filter);
	   let res = builder.build(&client).await?;

	   for event_data in res.events.iter().flatten() {
		   dbg!(event_data);
	   }

	   for call_data in res.calls.iter().flatten() {
		   dbg!(call_data);
	   }
	*/
	todo!();

	let s = client.clone();
	let t1 = tokio::spawn(async move { task(s, alice(), false).await });
	let s = client.clone();
	let t2 = tokio::spawn(async move { task(s, bob(), true).await });
	let s = client.clone();
	let t3 = tokio::spawn(async move { task(s, charlie(), true).await });
	let s = client.clone();
	let t4 = tokio::spawn(async move { task(s, dave(), true).await });

	t1.await.unwrap()?;
	t2.await.unwrap()?;
	t3.await.unwrap()?;
	t4.await.unwrap()?;

	Ok(())
}

async fn task(client: Client, account: Keypair, _d: bool) -> ReturnResult {
	// Data Submission
	let data = String::from("My Data").into_bytes();
	let options = Options::new().app_id(2);
	let tx = client.tx().data_availability.submit_data(data);

	// SubmittedTransaction -> Transaction Hash, and Transaction extra
	let st: SubmittedTransaction = tx.sign_and_submit(&account, options).await?;
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
