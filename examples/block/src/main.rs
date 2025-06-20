use avail_rust_client::{avail_rust_core::rpc::system::Filter, prelude::*};

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let hash = H256::from_str("0x78d4b201ec022555a81b8e9a070c8c0177ca6f5142d2ab1b178c3342bb6c0f7b").unwrap();

	let filter = Filter::Only(vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
	let a = client
		.rpc_api()
		.system_fetch_events_v1(Some(filter), false, false, hash)
		.await
		.unwrap();
	dbg!(a);
	Ok(())
}
