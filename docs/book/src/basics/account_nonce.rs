use avail_rust::{avail, error::ClientError, utils, Block, SDK};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = SDK::alice()?;
	let account_id = account.public_key().to_account_id();
	let account_address = account_id.to_string();

	// Fetch nonce from Node (this includes Tx Pool)
	let nonce = utils::fetch_nonce_node(&sdk.rpc_client, &account_address).await?;
	println!("Nonce from Node: {}", nonce);

	// Fetch nonce from best block state
	let nonce =
		utils::fetch_nonce_state(&sdk.online_client, &sdk.rpc_client, &account_address).await?;
	println!("Nonce from best block state: {}", nonce);

	// Fetch nonce from custom block state
	let block_hash = Block::fetch_finalized_block_hash(&sdk.rpc_client).await?;
	let block = sdk.online_client.blocks().at(block_hash).await?;
	let nonce = block.account_nonce(&account_id).await? as u32;
	println!("Nonce from custom block state: {}", nonce);

	// Fetch nonce from manually reading storage
	let storage = sdk.online_client.storage().at(block_hash);
	let address = avail::storage().system().account(account_id);
	let result = storage.fetch_or_default(&address).await?;
	println!("Nonce from  manually reading storage: {}", result.nonce);

	Ok(())
}

/*
	Expected Output:

	Nonce from Node: 1
	Nonce from best block state: 1
	Nonce from custom block state: 1
	Nonce from  manually reading storage: 1
*/
