use avail_rust_client::{block, error::Error, prelude::*};
use avail_rust_core::avail::{
	proxy::{
		events::{ProxyAdded, ProxyExecuted, ProxyRemoved, PureCreated},
		tx::{AddProxy, CreatePure, Proxy, RemoveProxy},
		types::ProxyType::{self},
	},
	system::types::{DispatchError, ModuleError},
};
use codec::Encode;

pub async fn run_tests() -> Result<(), Error> {
	tx_tests().await?;
	event_test().await?;

	Ok(())
}
pub async fn tx_tests() -> Result<(), Error> {
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Add Proxy
	{
		let block = block::SignedExtrinsics::new(client.clone(), 1076139);

		let id = "0xa6668ecbef4f8b0c64e294a9addc0fb267ec02cb0e0c3f74f3a45b8f1043c774";
		let submittable = client.tx().proxy().add_proxy(id, ProxyType::NonTransfer, 0);
		let expected_call = AddProxy::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<AddProxy>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Create Pure
	{
		let block = block::SignedExtrinsics::new(client.clone(), 1439619);

		let submittable = client.tx().proxy().create_pure(ProxyType::Any, 0, 0);
		let expected_call = CreatePure::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<CreatePure>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Proxy
	{
		let block = block::SignedExtrinsics::new(client.clone(), 1776412);

		let targets = vec![
			"0xc51d936c502bb72e4735619eeed59b3840cdbed6f414bb5da2b5bd977273d663",
			"0x3c243cc085dea34f4f2a1f40ad0740f1423aef957b5b35accc677cf2f4023130",
			"0x12ce9da1bfb72b90ae0060b2ce3ebc653b66d28e04f4821642dab6aefc9f5c2e",
			"0x209a04aa4a6eada5605b38d6bc87056e44a7c79fa31927ff73eb99df69329137",
			"0x0690d90d894580414030216c58faffc65e45b3257c264ffece9a6cf7369f1cb9",
			"0x3c0e5853201324a59630e80e15cd0049c637d1e68ae51a1d190e6f083263ad79",
			"0x4c86609864155fb79dd4939d4b5e09e5a8bd5032ca648a308575ecda7e182f72",
		];
		let c = client.tx().staking().nominate(targets);
		let id = "0xaabd39bb20728ec512a104178c2244703ae900eb3368ddcd3f8dbf6ed6151696";
		let submittable = client.tx().proxy().proxy(id, None, c);
		let expected_call = Proxy::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<Proxy>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	// Remove Proxy
	{
		let block = block::SignedExtrinsics::new(client.clone(), 790393);

		let delegate = "0x685302266408090333837daf4c1fee2b23c5a7f055b61f6e8d16ad6662b28b39";
		let submittable = client.tx().proxy().remove_proxy(delegate, ProxyType::Staking, 0);
		let expected_call = RemoveProxy::from_call(&submittable.call.encode()).unwrap();
		let actual_ext = block.get::<RemoveProxy>(1).await.unwrap().unwrap();
		assert_eq!(actual_ext.call.encode(), expected_call.encode());
	}

	Ok(())
}
pub async fn event_test() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// ProxyAdded
	{
		let events = block::Events::new(client.clone(), 2279940)
			.extrinsic(1)
			.await
			.unwrap()
			.unwrap();

		let expected = ProxyAdded {
			delegator: AccountId::from_str("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ").unwrap(),
			delegatee: AccountId::from_str("5H9Wh9UPU2kGZRCMLmEDKhhMxh1PLgBefMUgpLgGzFvjKkKw").unwrap(),
			proxy_type: ProxyType::Governance,
			delay: 25,
		};
		let actual = events.first::<ProxyAdded>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// PureCreated
	{
		let events = block::Events::new(client.clone(), 2279951)
			.extrinsic(1)
			.await
			.unwrap()
			.unwrap();

		let expected = PureCreated {
			pure: AccountId::from_str("5EYj7miFkQ8EFNbEdg7MfeG8dHKWHBoLXCrmoTXWZwMpmxAs").unwrap(),
			who: AccountId::from_str("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ").unwrap(),
			proxy_type: ProxyType::Any,
			disambiguation_index: 10,
		};
		let actual = events.first::<PureCreated>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// ProxyExecuted
	{
		let client = Client::new(MAINNET_ENDPOINT).await?;
		let events = block::Events::new(client.clone(), 1841067)
			.extrinsic(1)
			.await
			.unwrap()
			.unwrap();

		let expected = ProxyExecuted { result: Ok(()) };
		let actual = events.first::<ProxyExecuted>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// ProxyExecuted Failed
	{
		let events = block::Events::new(client.clone(), 2279971)
			.extrinsic(1)
			.await
			.unwrap()
			.unwrap();

		let expected = ProxyExecuted {
			result: Err(DispatchError::Module(ModuleError { index: 40, error: [1, 0, 0, 0] })),
		};
		let actual = events.first::<ProxyExecuted>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	// Proxy Removed
	{
		let events = block::Events::new(client.clone(), 2279990)
			.extrinsic(1)
			.await
			.unwrap()
			.unwrap();

		let expected = ProxyRemoved {
			delegator: AccountId::from_str("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ").unwrap(),
			delegatee: AccountId::from_str("5H9Wh9UPU2kGZRCMLmEDKhhMxh1PLgBefMUgpLgGzFvjKkKw").unwrap(),
			proxy_type: ProxyType::Any,
			delay: 0,
		};
		let actual = events.first::<ProxyRemoved>().unwrap();
		assert_eq!(actual.to_event(), expected.to_event());
	}

	Ok(())
}
