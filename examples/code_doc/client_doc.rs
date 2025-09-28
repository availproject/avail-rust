use avail_rust_client::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;
	let genesis_hash = client.online_client().genesis_hash();
	println!("You are connected to {}. Genesis Hash: {:?}", MAINNET_ENDPOINT, genesis_hash);
	/*
		You are connected to https://mainnet-rpc.avail.so/rpc. Genesis Hash: 0xb91746b45e0346cc2f815a520b9c6cb4d5c0902af848db0a80f85932d2e8276a
	*/

	Ok(())
}
