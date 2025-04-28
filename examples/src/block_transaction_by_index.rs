use avail_rust::prelude::*;

type TransferKeepAliveCall = avail::balances::calls::types::TransferKeepAlive;
type NetAccountEvent = avail::system::events::NewAccount;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;
	let block_hash = new_h256_from_hex("0x94746ba186876d7407ee618d10cb6619befc59eeb173cacb00c14d1ff492fc58")?;

	let block = Block::new(&sdk.client, block_hash).await?;

	// Transaction filtered by Transaction index
	let tx_index = 1;
	let txs = block.transactions(Filter::new().tx_index(tx_index));
	assert_eq!(txs.len(), 1, "");
	let tx = txs.get(0);

	// Printout
	assert_eq!(tx.tx_index(), tx_index, "Tx Index must be the same");
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

	// Convert from Block Transaction to Specific Transaction
	let ba_tx = tx.decode::<TransferKeepAliveCall>();
	assert!(ba_tx.is_some(), "TransferKeepAliveCall should exist");
	let ba_tx = ba_tx.unwrap();

	let account_id = match ba_tx.dest {
		subxt::utils::MultiAddress::Id(x) => x,
		_ => panic!("Not decodable."),
	};
	println!("Destination: {}, Value: {}", account_id, ba_tx.value);

	// Printout all Transaction Events
	let tx_events = tx.events().await;
	assert!(tx_events.is_some(), "Events should exist");
	let tx_events = tx_events.unwrap();
	assert_eq!(tx_events.len(), 9, "Event count must be 9");

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
	let event = tx_events.find_first::<NetAccountEvent>();
	assert!(event.as_ref().is_some_and(|x| x.is_some()), "NetAccountEvent");
	let event = event.unwrap().unwrap();
	println!("Account: {}", event.account);

	Ok(())
}
