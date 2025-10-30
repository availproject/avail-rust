use avail_rust_client::prelude::*;
use avail_rust_core::avail::proxy::types::ProxyType;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Error> {
	normal_proxy().await?;
	println!("");
	pure_proxy().await?;
	println!("");
	failed_proxy().await?;

	Ok(())
}

async fn normal_proxy() -> Result<(), Error> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let proxy_account = bob();
	let main_account = ferdie();

	// Creating Proxy
	let submittable = client
		.tx()
		.proxy()
		.add_proxy(proxy_account.account_id(), ProxyType::Any, 0);
	let submitted = submittable.sign_and_submit(&main_account, Default::default()).await?;

	let receipt = submitted.receipt(true).await?.expect("Should be there");
	let events = receipt.events().await?;
	assert_eq!(events.is_extrinsic_success_present(), true);

	let event = events
		.first::<avail::proxy::events::ProxyAdded>()
		.expect("Should be there");
	println!(
		"Delegatee: {}, Delegator: {}, ProxyType: {}, Delay: {}",
		event.delegatee, event.delegator, event.proxy_type, event.delay
	);

	// Executing Proxy::Proxy call
	let call = client
		.tx()
		.balances()
		.transfer_keep_alive(proxy_account.account_id(), ONE_AVAIL);
	let submittable = client.tx().proxy().proxy(main_account.account_id(), None, call);
	let submitted = submittable.sign_and_submit(&proxy_account, Default::default()).await?;

	let receipt = submitted.receipt(true).await?.expect("Should be there");
	let events = receipt.events().await?;
	assert_eq!(events.is_extrinsic_success_present(), true);
	assert_eq!(events.proxy_executed_successfully(), Some(true));

	let event = events
		.first::<avail::proxy::events::ProxyExecuted>()
		.expect("Should be there");
	println!("Result: {:?}", event.result);

	// Removing Proxy
	let submittable = client
		.tx()
		.proxy()
		.remove_proxy(proxy_account.account_id(), ProxyType::Any, 0);
	let submitted = submittable.sign_and_submit(&main_account, Default::default()).await?;

	let receipt = submitted.receipt(true).await?.expect("Should be there");
	let events = receipt.events().await?;
	assert_eq!(events.is_extrinsic_success_present(), true);

	let event = events
		.first::<avail::proxy::events::ProxyRemoved>()
		.expect("Should be there");
	println!(
		"Delegatee: {}, Delegator: {}, ProxyType: {}, Delay: {}",
		event.delegatee, event.delegator, event.proxy_type, event.delay
	);

	Ok(())
}

async fn pure_proxy() -> Result<(), Error> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let main_account = bob();

	let proxy_type = ProxyType::Any;
	let index = 0;

	// Creating Pure Proxy
	let submittable = client.tx().proxy().create_pure(proxy_type, 0, index);
	let submitted = submittable.sign_and_submit(&main_account, Default::default()).await?;

	let receipt = submitted.receipt(true).await?.expect("Should be there");
	let events = receipt.events().await?;
	assert_eq!(events.is_extrinsic_success_present(), true);

	let event = events
		.first::<avail::proxy::events::PureCreated>()
		.expect("Should be there");
	println!(
		"Pure: {}, Who: {}, Proxy Type: {}, index: {}",
		event.pure, event.who, event.proxy_type, event.disambiguation_index
	);
	let proxy_account_id = event.pure;

	// Executing Proxy::Proxy call
	let key: String = std::format!("{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
	let call = client.tx().data_availability().create_application_key(key);
	let submittable = client.tx().proxy().proxy(proxy_account_id.clone(), None, call);
	let submitted = submittable.sign_and_submit(&main_account, Default::default()).await?;

	let receipt = submitted.receipt(true).await?.expect("Should be there");
	let events = receipt.events().await?;
	assert_eq!(events.is_extrinsic_success_present(), true);
	assert_eq!(events.proxy_executed_successfully(), Some(true));

	let event = events
		.first::<avail::proxy::events::ProxyExecuted>()
		.expect("Should be there");
	println!("Result: {:?}", event.result);

	Ok(())
}

async fn failed_proxy() -> Result<(), Error> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let proxy_account = bob();
	let main_account = ferdie();

	// Creating Proxy
	let submittable = client
		.tx()
		.proxy()
		.add_proxy(proxy_account.account_id(), ProxyType::NonTransfer, 0);
	let submitted = submittable.sign_and_submit(&main_account, Default::default()).await?;

	let receipt = submitted.receipt(true).await?.expect("Should be there");
	let events = receipt.events().await?;
	assert_eq!(events.is_extrinsic_success_present(), true);

	let event = events
		.first::<avail::proxy::events::ProxyAdded>()
		.expect("Should be there");
	println!(
		"Delegatee: {}, Delegator: {}, ProxyType: {}, Delay: {}",
		event.delegatee, event.delegator, event.proxy_type, event.delay
	);

	// Executing Proxy::Proxy call
	let call = client
		.tx()
		.balances()
		.transfer_keep_alive(proxy_account.account_id(), ONE_AVAIL);
	let submittable = client.tx().proxy().proxy(main_account.account_id(), None, call);
	let submitted = submittable.sign_and_submit(&proxy_account, Default::default()).await?;

	let receipt = submitted.receipt(true).await?.expect("Should be there");
	let events = receipt.events().await?;
	assert_eq!(events.is_extrinsic_success_present(), true);
	assert_eq!(events.proxy_executed_successfully(), Some(false));

	let event = events
		.first::<avail::proxy::events::ProxyExecuted>()
		.expect("Should be there");
	println!("Result: {:?}", event.result);

	// Removing Proxy
	let submittable = client
		.tx()
		.proxy()
		.remove_proxy(proxy_account.account_id(), ProxyType::NonTransfer, 0);
	let submitted = submittable.sign_and_submit(&main_account, Default::default()).await?;

	let receipt = submitted.receipt(true).await?.expect("Should be there");
	let events = receipt.events().await?;
	assert_eq!(events.is_extrinsic_success_present(), true);

	let event = events
		.first::<avail::proxy::events::ProxyRemoved>()
		.expect("Should be there");
	println!(
		"Delegatee: {}, Delegator: {}, ProxyType: {}, Delay: {}",
		event.delegatee, event.delegator, event.proxy_type, event.delay
	);

	Ok(())
}

/*
	Expected Output:

	Delegatee: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Delegator: 5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL, ProxyType: Any, Delay: 0
	Result: Ok(())
	Delegatee: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Delegator: 5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL, ProxyType: Any, Delay: 0

	Pure: 5Es98N8wsxL3bFpfy79xzYFA5kZir57qXzPM1TDo5hkuA8mY, Who: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Proxy Type: Any, index: 0
	Result: Ok(())

	Delegatee: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Delegator: 5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL, ProxyType: NonTransfer, Delay: 0
	Result: Err(Module(ModuleError { index: 0, error: [5, 0, 0, 0] }))
	Delegatee: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, Delegator: 5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL, ProxyType: NonTransfer, Delay: 0
*/
