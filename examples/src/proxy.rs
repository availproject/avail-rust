use avail::{
	proxy::events::{ProxyAdded, ProxyExecuted, ProxyRemoved, PureCreated},
	runtime_types::{
		da_control::pallet::Call::create_application_key, pallet_balances::pallet::Call::transfer_keep_alive,
	},
};
use avail_rust::{
	avail::runtime_types::da_runtime::{impls::ProxyType, RuntimeCall},
	prelude::*,
};
use std::time::SystemTime;

pub async fn run() -> Result<(), ClientError> {
	run_normal_proxy().await?;
	run_pure_proxy().await?;
	run_proxy_failure().await?;

	println!("Proxy finished correctly");

	Ok(())
}

pub async fn run_normal_proxy() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let proxy_account = account::bob();
	let prox_account_multi = MultiAddress::from(proxy_account.public_key().to_account_id());
	let main_account = account::ferdie();
	let main_account_multi = MultiAddress::from(main_account.public_key().to_account_id());

	// Creating Proxy
	let proxy_type = ProxyType::Any;
	let tx = sdk
		.tx
		.proxy
		.add_proxy(prox_account_multi.clone(), proxy_type.clone(), 0);
	let res = tx.execute_and_watch_inclusion(&main_account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Finding Proxy Added Event
	let tx_events = res.events.as_ref().unwrap();
	let event = tx_events.find_first::<ProxyAdded>();
	let event = event.unwrap().unwrap();
	println!(
		"Delegatee: {}, Delegator: {}, ProxyTpe: {:?}, Delay: {}",
		event.delegatee, event.delegator, event.proxy_type, event.delay
	);

	// Executing the Proxy.Proxy() call
	let call = transfer_keep_alive {
		dest: prox_account_multi.clone(),
		value: SDK::one_avail(),
	};
	let call = RuntimeCall::Balances(call);

	let tx = sdk.tx.proxy.proxy(main_account_multi, None, call);
	let res = tx.execute_and_watch_inclusion(&proxy_account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Finding ProxyExecuted event
	let tx_events = res.events.as_ref().unwrap();
	let event = tx_events.find_first::<ProxyExecuted>();
	let event = event.unwrap().unwrap();
	assert!(event.result.is_ok());

	// Removing Proxy
	let tx = sdk.tx.proxy.remove_proxy(prox_account_multi, proxy_type, 0);
	let res = tx.execute_and_watch_inclusion(&main_account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Finding for EventProxyRemoved event.
	let tx_events = res.events.as_ref().unwrap();
	let event = tx_events.find_first::<ProxyRemoved>();
	let event = event.unwrap().unwrap();
	println!(
		"Delegatee: {}, Delegator: {}, ProxyTpe: {:?}, Delay: {}",
		event.delegatee, event.delegator, event.proxy_type, event.delay
	);

	Ok(())
}

pub async fn run_pure_proxy() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let main_account = account::ferdie();

	// Creating Proxy
	let proxy_type = ProxyType::Any;
	let tx = sdk.tx.proxy.create_pure(proxy_type.clone(), 0, 0);
	let res = tx.execute_and_watch_inclusion(&main_account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Finding PureCreated Event
	let tx_events = res.events.as_ref().unwrap();
	let event = tx_events.find_first::<PureCreated>();
	let event = event.unwrap().unwrap();
	println!(
		"Pure: {}, Who: {}, ProxyTpe: {:?}, Disambiguation Index: {}",
		event.pure, event.who, event.proxy_type, event.disambiguation_index
	);
	let pure_proxy = event.pure;
	let pure_proxy_multi = MultiAddress::from(pure_proxy);

	// Executing the Proxy.Proxy() call
	let time = std::format!("{:?}", SystemTime::now());
	let key = time.into_bytes();
	let call = create_application_key { key: BoundedVec(key) };
	let call = RuntimeCall::DataAvailability(call);

	let tx = sdk.tx.proxy.proxy(pure_proxy_multi, None, call);
	let res = tx.execute_and_watch_inclusion(&main_account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Finding ProxyExecuted event
	let tx_events = res.events.as_ref().unwrap();
	let event = tx_events.find_first::<ProxyExecuted>();
	let event = event.unwrap().unwrap();
	assert!(event.result.is_ok());

	Ok(())
}

pub async fn run_proxy_failure() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let proxy_account = account::bob();
	let prox_account_multi = MultiAddress::from(proxy_account.public_key().to_account_id());
	let main_account = account::ferdie();
	let main_account_multi = MultiAddress::from(main_account.public_key().to_account_id());

	// Creating Proxy
	let proxy_type = ProxyType::NonTransfer;
	let tx = sdk
		.tx
		.proxy
		.add_proxy(prox_account_multi.clone(), proxy_type.clone(), 0);
	let res = tx.execute_and_watch_inclusion(&main_account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Executing the Proxy.Proxy() call
	let call = transfer_keep_alive {
		dest: prox_account_multi.clone(),
		value: SDK::one_avail(),
	};
	let call = RuntimeCall::Balances(call);

	let tx = sdk.tx.proxy.proxy(main_account_multi, None, call);
	let res = tx.execute_and_watch_inclusion(&proxy_account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));

	// Finding ProxyExecuted event
	let tx_events = res.events.as_ref().unwrap();
	let event = tx_events.find_first::<ProxyExecuted>();
	let event = event.unwrap().unwrap();
	assert!(event.result.is_err());
	println!("Proxy error: {:?}", event.result.unwrap_err());

	// Removing Proxy
	let tx = sdk.tx.proxy.remove_proxy(prox_account_multi, proxy_type, 0);
	let res = tx.execute_and_watch_inclusion(&main_account, Options::new()).await?;
	assert_eq!(res.is_successful(), Some(true));
	Ok(())
}
