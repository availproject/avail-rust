use std::ops::Mul;

use avail_rust::prelude::*;

use avail::{
	runtime_types::{da_runtime::RuntimeCall, pallet_balances::pallet::Call::transfer_keep_alive as TransferKeepAlive},
	utility::events as UtilityEvents,
};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = account::alice();

	let value_1 = SDK::one_avail();
	let value_2 = SDK::one_avail();
	let dest_bob = account::account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?;
	let dest_charlie = account::account_id_from_str("5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y")?;

	let call_1 = TransferKeepAlive {
		dest: dest_bob.clone().into(),
		value: value_1,
	};
	let call_2 = TransferKeepAlive {
		dest: dest_charlie.clone().into(),
		value: value_2,
	};
	let mut calls = Vec::new();
	calls.push(RuntimeCall::Balances(call_1).into());
	calls.push(RuntimeCall::Balances(call_2).into());

	//
	// Happy Path
	//

	// Batch Call
	//
	// This will return `Ok` in all circumstances. To determine the success of the batch, an
	// event is deposited. If a call failed and the batch was interrupted, then the
	// `BatchInterrupted` event is deposited, along with the number of successful calls made
	// and the error of the failed call. If all were successful, then the `BatchCompleted`
	// event is deposited.
	let payload = avail::tx().utility().batch(calls.clone());
	let tx = Transaction::new(sdk.client.clone(), payload);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions needs to be successful");

	let event = res.find_first_event::<UtilityEvents::BatchCompleted>();
	assert!(event.is_some(), "BatchCompleted event must be present.");

	let event_count = res.find_event::<UtilityEvents::ItemCompleted>().len();
	assert_eq!(event_count, 2, "ItemCompleted events must be produced twice");

	println!("-- Batch Call Done --");

	// Batch All Call
	//
	// Send a batch of dispatch calls and atomically execute them.
	// The whole transaction will rollback and fail if any of the calls failed.
	let payload = avail::tx().utility().batch_all(calls.clone());
	let tx = Transaction::new(sdk.client.clone(), payload);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions needs to be successful");

	let event = res.find_first_event::<UtilityEvents::BatchCompleted>();
	assert!(event.is_some(), "BatchCompleted event must be present.");

	let event_count = res.find_event::<UtilityEvents::ItemCompleted>().len();
	assert_eq!(event_count, 2, "ItemCompleted events must be produced twice");

	println!("-- Batch All Call Done --");

	// Force Batch Call
	//
	// Send a batch of dispatch calls.
	// Unlike `batch`, it allows errors and won't interrupt.

	let payload = avail::tx().utility().force_batch(calls.clone());
	let tx = Transaction::new(sdk.client.clone(), payload);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions needs to be successful");

	let event = res.find_first_event::<UtilityEvents::BatchCompleted>();
	assert!(event.is_some(), "BatchCompleted event must be present.");

	let event_count = res.find_event::<UtilityEvents::ItemCompleted>().len();
	assert_eq!(event_count, 2, "ItemCompleted events must be produced twice");

	println!("-- Force Batch Call Done --");

	//
	//	Things differ when we introduce a call that will fail
	//

	let call_3 = TransferKeepAlive {
		dest: dest_charlie.into(),
		value: SDK::one_avail().mul(1_000_000_000u128),
	};

	let call_4 = TransferKeepAlive {
		dest: dest_bob.into(),
		value: SDK::one_avail().mul(1u128),
	};

	// The 3. is poisoned with a too high transfer amount
	calls.push(RuntimeCall::Balances(call_3).into());
	// The 4. call is a normal one
	calls.push(RuntimeCall::Balances(call_4).into());

	// Batch Call
	let payload = avail::tx().utility().batch(calls.clone());
	let tx = Transaction::new(sdk.client.clone(), payload);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions needs to be successful");

	let event = res.find_first_event::<UtilityEvents::BatchInterrupted>();
	assert!(event.is_some(), "BatchInterrupted event must be present.");

	let event = res.find_first_event::<UtilityEvents::BatchCompleted>();
	assert!(event.is_none(), "BatchCompleted event must NOT be present.");

	let event_count = res.find_event::<UtilityEvents::ItemCompleted>().len();
	assert_eq!(event_count, 2, "ItemCompleted events must be produced twice");

	println!("-- Batch Call Done --");

	// Batch All Call
	let payload = avail::tx().utility().batch_all(calls.clone());
	let tx = Transaction::new(sdk.client.clone(), payload);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	assert_eq!(res.is_successful(), Some(false), "Transactions needs fail.");

	println!("-- Batch All Call Done --");

	// Force Batch Call
	let payload = avail::tx().utility().force_batch(calls.clone());
	let tx = Transaction::new(sdk.client.clone(), payload);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions needs to be successful");

	let event = res.find_first_event::<UtilityEvents::BatchCompletedWithErrors>();
	assert!(event.is_some(), "BatchCompletedWithErrors event must be present.");

	let event_count = res.find_event::<UtilityEvents::ItemCompleted>().len();
	assert_eq!(event_count, 3, "ItemCompleted events must be produced thrice");

	let event_count = res.find_event::<UtilityEvents::ItemFailed>().len();
	assert_eq!(event_count, 1, "ItemFailed events must be produced once");

	println!("-- Force Batch Call Done --");

	println!("Batch finished correctly");

	Ok(())
}
