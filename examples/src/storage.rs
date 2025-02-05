use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	println!("da_app_keys");
	da_app_keys().await?;
	println!("da_app_keys_iter");
	da_app_keys_iter().await?;
	println!("da_next_app_id");
	da_next_app_id().await?;
	println!("staking_active_era");
	staking_active_era().await?;
	println!("staking_bonded");
	staking_bonded().await?;
	println!("staking_bonded_iter");
	staking_bonded_iter().await?;
	println!("system_account");
	system_account().await?;
	println!("system_account_iter");
	system_account_iter().await?;

	println!("Storage finished correctly");

	Ok(())
}

pub async fn da_app_keys() -> Result<(), ClientError> {
	use avail::data_availability::storage::types::app_keys::Param0;

	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let key = String::from("Reserved-1").as_bytes().to_vec();
	let key = Param0 { 0: key };

	let block_hash = sdk.client.best_block_hash().await?;
	let storage = sdk.client.storage().at(block_hash);
	let address = avail::storage().data_availability().app_keys(key);
	let result = storage.fetch(&address).await?;

	dbg!(result);
	/* Output
	AppKeyInfo {
		owner: AccountId32(...),
		id: AppId(
			1,
		),
	}
	*/

	Ok(())
}

pub async fn da_app_keys_iter() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let block_hash = sdk.client.best_block_hash().await?;
	let storage = sdk.client.storage().at(block_hash);
	let address = avail::storage().data_availability().app_keys_iter();
	let mut results = storage.iter(address).await?;

	while let Some(Ok(kv)) = results.next().await {
		let key = (&kv.key_bytes[49..]).to_vec();
		let key = String::from_utf8(key).unwrap();

		println!("Key: {:?}", key);
		println!("Value: {:?}", kv.value);
	}
	/* Output
		Key: "Reserved-2"
		Value: AppKeyInfo { owner: AccountId32(...), id: AppId(2) }
		Key: "Reserved-8"
		Value: AppKeyInfo { owner: AccountId32(...), id: AppId(8) }
		Key: "Reserved-1"
		Value: AppKeyInfo { owner: AccountId32(...) id: AppId(1) }
		Key: "Reserved-9"
		Value: AppKeyInfo { owner: AccountId32(...), id: AppId(9) }
		Key: "Reserved-4"
		Value: AppKeyInfo { owner: AccountId32(...), id: AppId(4) }
		Key: "Reserved-5"
		Value: AppKeyInfo { owner: AccountId32(...), id: AppId(5) }
		Key: "Reserved-7"
		Value: AppKeyInfo { owner: AccountId32(...), id: AppId(7) }
		Key: "Avail"
		Value: AppKeyInfo { owner: AccountId32(...), id: AppId(0) }
		Key: "Reserved-3"
		Value: AppKeyInfo { owner: AccountId32(...), id: AppId(3) }
		Key: "Reserved-6"
		Value: AppKeyInfo { owner: AccountId32(...), id: AppId(6) }
	*/

	Ok(())
}

pub async fn da_next_app_id() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let block_hash = sdk.client.best_block_hash().await?;
	let storage = sdk.client.storage().at(block_hash);
	let address = avail::storage().data_availability().next_app_id();
	let result = storage.fetch_or_default(&address).await?;

	dbg!(result);
	/* Output
		AppId(10)
	*/

	Ok(())
}

pub async fn staking_active_era() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let block_hash = sdk.client.best_block_hash().await?;
	let storage = sdk.client.storage().at(block_hash);
	let address = avail::storage().staking().active_era();
	let result = storage.fetch(&address).await?;

	dbg!(result);
	/* Output
	ActiveEraInfo {
		index: 13,
		start: Some(
			1732612788000,
		),
	}
	*/

	Ok(())
}

pub async fn staking_bonded() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account_id = account::account_id_from_str("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY")?; // Alice_Stash

	let block_hash = sdk.client.best_block_hash().await?;
	let storage = sdk.client.storage().at(block_hash);
	let address = avail::storage().staking().bonded(account_id);
	let result = storage.fetch(&address).await?;

	dbg!(result);
	/* Output
		AccountId32(...)
	*/

	Ok(())
}

pub async fn staking_bonded_iter() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let block_hash = sdk.client.best_block_hash().await?;
	let storage = sdk.client.storage().at(block_hash);
	let storage_query = avail::storage().staking().bonded_iter();
	let mut results = storage.iter(storage_query).await?;

	while let Some(Ok(kv)) = results.next().await {
		let key = kv.key_bytes.last_chunk::<32>().unwrap();
		let key = AccountId::from(*key);

		println!("Key: {:?}", key.to_string());
		println!("Value: {:?}", kv.value);
	}
	/* Output
		Key: "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY"
		Value: AccountId32(...)
	*/

	Ok(())
}

pub async fn system_account() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = account::alice();
	let account_id = account.public_key().to_account_id();

	let block_hash = sdk.client.best_block_hash().await?;
	let storage = sdk.client.storage().at(block_hash);
	let address = avail::storage().system().account(account_id);
	let result = storage.fetch(&address).await?;

	if let Some(account) = result {
		println!("Consumers: {}", account.consumers);
		println!("Data: {:?}", account.data);
		println!("Nonce: {}", account.nonce);
		println!("Providers: {}", account.providers);
		println!("Sufficients: {}", account.sufficients);
	}
	/* Output
		Consumers: 0
		Data: AccountData { free: 10000000000000000000000000, reserved: 0, frozen: 0, flags: ExtraFlags(170141183460469231731687303715884105728) }
		Nonce: 0
		Providers: 1
		Sufficients: 0
	*/

	Ok(())
}

pub async fn system_account_iter() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let block_hash = sdk.client.best_block_hash().await?;
	let storage = sdk.client.storage().at(block_hash);
	let address = avail::storage().system().account_iter();
	let mut results = storage.iter(address).await?;

	while let Some(Ok(kv)) = results.next().await {
		let key = kv.key_bytes.last_chunk::<32>().unwrap();
		let key = AccountId::from(*key);

		println!("Key: {:?}", key.to_string());
		println!("Value: {:?}", kv.value);
	}
	/* Output
		Key: "5FCfAonRZgTFrTd9HREEyeJjDpT397KMzizE6T3DvebLFE7n"
		Value: AccountInfo { nonce: 0, consumers: 0, providers: 1, sufficients: 0, data: AccountData { free: 10000000000000000000000000, reserved: 0, frozen: 0, flags: ExtraFlags(170141183460469231731687303715884105728) } }

		Key: "5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL"
		Value: AccountInfo { nonce: 0, consumers: 0, providers: 1, sufficients: 0, data: AccountData { free: 10000000000000000000000000, reserved: 0, frozen: 0, flags: ExtraFlags(170141183460469231731687303715884105728) } }

		Key: "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY"
		Value: AccountInfo { nonce: 0, consumers: 3, providers: 1, sufficients: 0, data: AccountData { free: 10000001075151923366255874, reserved: 0, frozen: 100000000000000000000000, flags: ExtraFlags(170141183460469231731687303715884105728) } }

		Key: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
		Value: AccountInfo { nonce: 0, consumers: 0, providers: 1, sufficients: 0, data: AccountData { free: 10000000000000000000000000, reserved: 0, frozen: 0, flags: ExtraFlags(170141183460469231731687303715884105728) } }
		...
	*/

	Ok(())
}
