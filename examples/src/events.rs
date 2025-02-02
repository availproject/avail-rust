use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = SDK::alice()?;

	let dest = account::account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?;
	let tx = sdk.tx.balances.transfer_keep_alive(dest, SDK::one_avail());
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	let events = match &res.events {
		Some(x) => x,
		None => panic!("Failed to decode events."),
	};

	for event in events.iter() {
		let Ok(event) = event else {
			return Ok(());
		};

		println!(
			"Pallet name: {}, Event Name: {}",
			event.pallet_name(),
			event.variant_name()
		);
	}

	// find_first_event, find_last_event, find_event
	let event = res.find_first_event::<avail::balances::events::Transfer>();
	let Some(event) = event else {
		return Ok(());
	};

	println!(
		"Transfer from: {}, to: {}, amount: {}",
		event.from, event.to, event.amount
	);

	Ok(())
}

/*
	Example Output:

	Pallet name: Balances, Event Name: Withdraw
	Pallet name: Balances, Event Name: Transfer
	Pallet name: Balances, Event Name: Deposit
	Pallet name: Balances, Event Name: Deposit
	Pallet name: Balances, Event Name: Deposit
	Pallet name: TransactionPayment, Event Name: TransactionFeePaid
	Pallet name: System, Event Name: ExtrinsicSuccess
	Transfer from: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY, to: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, amount: 1000000000000000000
*/
