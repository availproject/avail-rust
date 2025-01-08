use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = SDK::alice()?;

	let dest = account::account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?;
	let tx = sdk.tx.balances.transfer_keep_alive(dest, SDK::one_avail());
	let res = tx.execute_and_watch_inclusion(&account, None).await?;

	let block = Block::new(&sdk.online_client, res.block_hash).await?;

	// transaction_all_static, transaction_count, transaction_by_signer, transaction_by_signer_static
	// transaction_by_index, transaction_by_index_static, transaction_by_hash,
	// transaction_by_hash_static, transaction_by_app_id, transaction_by_app_id_static
	for tx in block.transactions.iter() {
		println!(
			"Tx Pallet name: {}, Tx Name: {}, Tx Hash: {:?}",
			tx.pallet_name()?,
			tx.variant_name()?,
			tx.hash()
		);

		for event in tx.events().await?.iter() {
			let Ok(event) = event else {
				return Ok(());
			};

			println!(
				"\tEvent Pallet name: {}, Event Name: {}",
				event.pallet_name(),
				event.variant_name()
			);
		}

		let balance_tx = tx.as_extrinsic::<avail::balances::calls::types::TransferKeepAlive>();
		if let Some(tx) = balance_tx.ok().flatten() {
			println!("Transfer dest: {:?}, value: {}", tx.dest, tx.value);
		}
	}

	// Transaction object can be used with custom payload.
	// ! Check Transaction 1(basics_1) or Transaction 2(basics_2) example for custom payload. !
	let dest = account::account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?;
	let payload = avail_rust::avail::tx()
		.balances()
		.transfer_keep_alive(dest.into(), SDK::one_avail());
	let tx = Transaction::new(sdk.online_client.clone(), sdk.rpc_client.clone(), payload);
	let res = tx.execute_and_watch_inclusion(&account, None).await?;
	// Checking if the transaction was successful
	match res.is_successful(&sdk.online_client) {
		Some(x) => x?,
		None => panic!("Failed to decode events."),
	};

	Ok(())
}

/*
	Example Output:

	Tx Pallet name: Timestamp, Tx Name: set, Tx Hash: 0xdf4e9c7ae69b40936b580ddf2d7c9b0cf5adb55e64f8492d1e160cc0914a8889
		Event Pallet name: System, Event Name: ExtrinsicSuccess
	Tx Pallet name: Balances, Tx Name: transfer_keep_alive, Tx Hash: 0x748057951ff79cea6de0e13b2ef70a1e9f443e9c83ed90e5601f8b45144a4ed4
		Event Pallet name: Balances, Event Name: Withdraw
		Event Pallet name: Balances, Event Name: Transfer
		Event Pallet name: Balances, Event Name: Deposit
		Event Pallet name: Balances, Event Name: Deposit
		Event Pallet name: Balances, Event Name: Deposit
		Event Pallet name: TransactionPayment, Event Name: TransactionFeePaid
		Event Pallet name: System, Event Name: ExtrinsicSuccess
	Transfer dest: Id(AccountId32([142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72])), value: 1000000000000000000
	Tx Pallet name: Vector, Tx Name: failed_send_message_txs, Tx Hash: 0x92cdb77314063a01930b093516d19a453399710cc8ae635ff5ab6cf76b26f218
		Event Pallet name: System, Event Name: ExtrinsicSuccess
*/
