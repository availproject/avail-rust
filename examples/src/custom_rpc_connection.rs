/* use avail_rust::prelude::*;
use std::time::Duration;
use subxt::backend::rpc::{
	reconnecting_rpc_client::{ExponentialBackoff, RpcClient as ReconnectingRpcClient},
	RpcClient,
};

pub async fn run() -> Result<(), ClientError> {
	let rpc_client = ReconnectingRpcClient::builder()
		.retry_policy(
			ExponentialBackoff::from_millis(1000)
				.max_delay(Duration::from_secs(3))
				.take(3),
		)
		.build(SDK::local_endpoint())
		.await
		.map_err(|e| e.to_string())?;
	let rpc_client = RpcClient::new(rpc_client);
	let online_client = AOnlineClient::from_rpc_client(rpc_client.clone()).await?;
	let client = Client::new(online_client, rpc_client);

	let _sdk = SDK::new_custom(client).await?;

	Ok(())
}
 */
