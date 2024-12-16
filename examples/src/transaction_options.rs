use avail_rust::prelude::*;
use std::time::Duration;

pub async fn run() -> Result<(), ClientError> {
	nonce().await?;
	app_id().await?;
	tip().await?;
	mortality().await?;

	Ok(())
}

async fn nonce() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = SDK::alice()?;
	let account_address = account.public_key().to_account_id().to_string();
	let dest = account::account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?;
	let mut options = Options::new();
	let tx = sdk.tx.balances.transfer_keep_alive(dest, SDK::one_avail());

	/*
		Using finalized block nonce will not take into consideration nonces from non-finalized blocks.
	*/
	options = options.nonce(Nonce::FinalizedBlock);
	tx.execute(&account, Some(options)).await?;
	tx.execute(&account, Some(options)).await.expect_err("qed");
	wait_n_blocks(&sdk, 3).await?;

	/*
		Using best block nonce will not take into consideration existing transactions in the
		tx pool.
	*/
	options = options.nonce(Nonce::BestBlock);
	tx.execute(&account, Some(options)).await?;
	tx.execute(&account, Some(options)).await.expect_err("qed");
	wait_n_blocks(&sdk, 1).await?;

	/*
		This is the most commonly used nonce. If correctness is needed, use `Nonce::FinalizedBlock`
		This is the default behavior,
	*/
	options = options.nonce(Nonce::BestBlockAndTxPool);
	tx.execute(&account, Some(options)).await?;
	tx.execute(&account, Some(options)).await?;

	/*
		Managing the nonce manually
	*/
	let nonce = account::fetch_nonce_node(&sdk.rpc_client, &account_address).await?;

	options = options.nonce(Nonce::Custom(nonce));
	tx.execute(&account, Some(options)).await?;
	options = options.nonce(Nonce::Custom(nonce + 1));
	tx.execute(&account, Some(options)).await?;

	Ok(())
}

async fn app_id() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = SDK::alice()?;
	let tx = sdk.tx.data_availability.submit_data(vec![0, 1, 2]);

	let options = Options::new().app_id(1);
	tx.execute(&account, Some(options)).await?;

	Ok(())
}

async fn tip() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = SDK::alice()?;
	let dest = account::account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?;
	let tx = sdk.tx.balances.transfer_keep_alive(dest, SDK::one_avail());

	let options = Options::new().tip(1);
	tx.execute(&account, Some(options)).await?;

	Ok(())
}

async fn mortality() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = SDK::alice()?;
	let dest = account::account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")?;
	let tx = sdk.tx.balances.transfer_keep_alive(dest, SDK::one_avail());

	let period = 3;
	let block_hash = None;
	let mortality = Mortality::new(period, block_hash);

	let options = Options::new().mortality(mortality);
	tx.execute(&account, Some(options)).await?;

	Ok(())
}

async fn wait_n_blocks(sdk: &SDK, n: u32) -> Result<(), ClientError> {
	let mut expected_block_number = None;

	loop {
		let current_block = rpc::chain::get_block(&sdk.rpc_client, None).await?;
		let current_block_number = current_block.block.header.number;
		if expected_block_number.is_none() {
			expected_block_number = Some(current_block_number + n);
		}
		if expected_block_number.is_some_and(|x| x <= current_block_number) {
			break;
		}

		tokio::time::sleep(Duration::from_secs(3)).await
	}

	Ok(())
}
