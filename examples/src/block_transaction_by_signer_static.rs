use avail_rust::prelude::*;

type CreateAppKeyCall = avail::data_availability::calls::types::CreateApplicationKey;
type AppKeyCreated = avail::data_availability::events::ApplicationKeyCreated;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;
	let block_hash = new_h256_from_hex("0x94746ba186876d7407ee618d10cb6619befc59eeb173cacb00c14d1ff492fc58")?;

	let block = Block::new(&sdk.client, block_hash).await?;

	// All Transaction filtered by Signer
	let account_id = account_id_from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")?;
	let block_transactions = block.transactions_static::<CreateAppKeyCall>(Filter::new().tx_signer(account_id.clone()));
	assert_eq!(block_transactions.len(), 1, "Transaction count must be 1");

	// Printout Block Transactions made by Signer
	for tx in block_transactions.iter() {
		assert_eq!(tx.account_id(), Some(account_id.clone()), "Signer must be the same");

		let key = to_ascii(tx.value.key.0.clone()).unwrap();
		println!("Key: {}", key);

		println!(
			"Pallet Name: {:?}, Pallet Index: {}, Call Name: {:?}, Call Index: {:?}, Tx Hash: {:?}, Tx Index: {}",
			tx.pallet_name(),
			tx.pallet_index(),
			tx.call_name(),
			tx.call_index(),
			tx.tx_hash(),
			tx.tx_index()
		);

		println!(
			"Tx Signer: {:?}, App Id: {:?}, Tip: {:?}, Mortality: {:?}, Nonce: {:?}",
			tx.ss58address(),
			tx.app_id(),
			tx.tip(),
			tx.mortality(),
			tx.nonce(),
		);
	}

	// Printout all Transaction Events
	let tx_events = block_transactions[0].events().await;
	assert!(tx_events.is_some(), "Events should exist");
	let tx_events = tx_events.unwrap();
	assert_eq!(tx_events.len(), 7, "Event count must be 7");

	for event in tx_events.iter() {
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

	// Convert from Generic Transaction Event to Specific Transaction Event
	let event = tx_events.find_first::<AppKeyCreated>();
	assert!(event.as_ref().is_some_and(|x| x.is_some()), "AppKeyCreated");
	let event = event.unwrap().unwrap();

	let key = to_ascii(event.key.0).unwrap();
	println!("App Id: {}, Owner: {}, Key: {}", event.id.0, event.owner, key);

	Ok(())
}
