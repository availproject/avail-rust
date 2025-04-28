use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	run_app_id().await?;
	run_nonce().await?;
	run_tip().await?;
	run_mortality().await?;

	println!("Transaction Options finished correctly");

	Ok(())
}

async fn run_app_id() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	// Executing Transaction
	let app_id = 5u32;
	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);
	let options = Options::new().app_id(app_id);
	let res = tx.execute_and_watch(&account::alice(), options).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Check if the correct app id has been used
	let block = Block::new(&sdk.client, res.block_hash).await?;
	let block_txs = block.transactions(Filter::new().tx_hash(res.tx_hash));
	assert_eq!(block_txs.len(), 1);
	assert_eq!(block_txs.get(0).app_id(), Some(app_id));

	println!("Transaction Options App Id finished correctly");

	Ok(())
}

async fn run_nonce() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;
	let account = account::alice();

	// Executing Transaction
	let nonce = account::nonce(&sdk.client, &std::format!("{}", account.public_key().to_account_id())).await?;
	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);
	let options = Options::new().nonce(nonce);
	let res = tx.execute_and_watch(&account, options).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Check if the correct app id has been used
	let block = Block::new(&sdk.client, res.block_hash).await?;
	let block_txs = block.transactions(Filter::new().tx_hash(res.tx_hash));
	assert_eq!(block_txs.len(), 1);
	assert_eq!(block_txs.get(0).nonce(), Some(nonce));

	println!("Transaction Options Nonce finished correctly");

	Ok(())
}

async fn run_tip() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	// Executing Transaction
	let tip = SDK::one_avail();
	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);
	let options = Options::new().tip(tip);
	let res = tx.execute_and_watch(&account::alice(), options).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Check if the correct app id has been used
	let block = Block::new(&sdk.client, res.block_hash).await?;
	let block_txs = block.transactions(Filter::new().tx_hash(res.tx_hash));
	assert_eq!(block_txs.len(), 1);
	assert_eq!(block_txs.get(0).tip(), Some(tip));

	println!("Transaction Options Tip finished correctly");

	Ok(())
}

async fn run_mortality() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	// Executing Transaction
	let mortality = 8u64;
	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);
	let options = Options::new().mortality(mortality);
	let res = tx.execute_and_watch(&account::alice(), options).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Check if the correct app id has been used
	let block = Block::new(&sdk.client, res.block_hash).await?;
	let block_txs = block.transactions(Filter::new().tx_hash(res.tx_hash));
	assert_eq!(block_txs.len(), 1);

	let actual_mortality = block_txs.get(0).mortality().unwrap();
	let actual_mortality = match actual_mortality {
		subxt::utils::Era::Mortal { period, phase: _ } => period,
		_ => panic!("Should not be here"),
	};

	assert_eq!(actual_mortality, mortality);

	println!("Transaction Options Mortality finished correctly");

	Ok(())
}
