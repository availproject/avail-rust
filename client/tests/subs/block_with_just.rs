use crate::subs::client_mock::MockClient;
use avail_rust_client::{error::Error, prelude::*, subscription::BlockWithJustSub, subxt_rpcs::RpcClient};

pub async fn run_tests() -> Result<(), Error> {
	let rpc_client = RpcClient::new(MockClient::new(MAINNET_ENDPOINT));
	let client = Client::new_rpc_client(rpc_client).await?;

	// Historical block
	let mut sub = BlockWithJustSub::new(client.clone());
	sub.set_block_height(1908729);

	let block = sub.next().await?.unwrap();
	assert_eq!(block.block.header.number, 1908729);

	let block = sub.next().await?.unwrap();
	assert_eq!(block.block.header.number, 1908730);

	// Best Block
	let expected = client.best().block_height().await?;
	let mut sub = BlockWithJustSub::new(client.clone());
	sub.set_follow(true);

	let block = sub.next().await?.unwrap();
	assert_eq!(block.block.header.number, expected);

	// Finalized Block
	let expected = client.finalized().block_height().await?;
	let mut sub = BlockWithJustSub::new(client);
	sub.set_follow(false);

	let block = sub.next().await?.unwrap();
	assert_eq!(block.block.header.number, expected);

	Ok(())
}
