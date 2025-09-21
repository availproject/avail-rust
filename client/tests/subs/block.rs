use crate::subs::client_mock::MockClient;
use avail_rust_client::{error::Error, prelude::*, subscription::BlockSub, subxt_rpcs::RpcClient};

pub async fn run_tests() -> Result<(), Error> {
	let rpc_client = RpcClient::new(MockClient::new(MAINNET_ENDPOINT));
	let client = Client::new_rpc_client(rpc_client).await?;

	// Historical block
	let mut sub = BlockSub::new(client.clone());
	sub.set_block_height(1908729);

	let (_, info) = sub.next().await?;
	assert_eq!(info.height, 1908729);

	let (_, info) = sub.next().await?;
	assert_eq!(info.height, 1908730);

	// Best Block
	let expected = client.best().block_height().await?;
	let mut sub = BlockSub::new(client.clone());
	sub.set_follow(true);

	let (_, info) = sub.next().await?;
	assert_eq!(info.height, expected);

	// Finalized Block
	let expected = client.finalized().block_height().await?;
	let mut sub = BlockSub::new(client);
	sub.set_follow(false);

	let (_, info) = sub.next().await?;
	assert_eq!(info.height, expected);

	Ok(())
}
