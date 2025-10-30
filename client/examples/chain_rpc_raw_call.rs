use avail_rust_client::{prelude::*, subxt_rpcs::client::RpcParams};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// RPC Raw call
	let mut params = RpcParams::new();
	params.push(2000000u32).expect("Should work");

	let block_hash = client
		.chain()
		.rpc_raw_call::<Option<H256>>("chain_getBlockHash", params)
		.await?;
	let block_hash = block_hash.expect("Should exist");
	println!("chain_getBlockHash: Block Hash: {:?}", block_hash);

	Ok(())
}

/*
	Expected Output:

	chain_getBlockHash: Block Hash: 0x6831d536cc3d6408a41a1e50d66f4f48c9c2ed5ffc7cfa7505a5f0251365428f
*/
