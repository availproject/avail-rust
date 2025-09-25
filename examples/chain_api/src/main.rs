use avail_rust::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	// Establishing a connection
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// Block Hash
	let best = client.best().block_hash().await?;
	let finalized = client.finalized().block_hash().await?;
	let block_hash = client.rpc().block_hash(Some(1900000)).await?.expect("Should be there");
	println!("Best: {:?}, Finalized: {:?}, Specific: {:?}", best, finalized, block_hash);
	/*
		Best: 	   0x1f1c30e3327487c5f0f2562fd4d6023f3582d30b3cd4f26e23304f845437732a,
		Finalized: 0xb5e82270ce67e77be50521768f8ebb6db7c5ea020f8ed611f13d994dad361af9,
		Specific:  0x6352f7d9cc7ac541170e769b8b7c0f0bcb22ebd6ba80bcd1313e265c33c911c7
	*/

	// Block Height
	let best = client.best().block_height().await?;
	let finalized = client.finalized().block_height().await?;
	let block_height = client.rpc().block_height(block_hash).await?.expect("Should be there");
	println!("Best: {}, Finalized: {}, Specific: {}", best, finalized, block_height);
	/*
		Best: 	   1922556,
		Finalized: 1922554,
		Specific:  1900000
	*/

	// Block Info
	let best = client.best().block_info().await?;
	let finalized = client.finalized().block_info().await?;
	println!("Best Hash: {:?}, Height: {}", best.hash, best.height);
	println!("Finalized Hash: {:?}, Height: {}", finalized.hash, finalized.height);
	/*
		Best 	  Hash: 0x1f1c30e3327487c5f0f2562fd4d6023f3582d30b3cd4f26e23304f845437732a, Height:	1922556
		Finalized Hash: 0xb5e82270ce67e77be50521768f8ebb6db7c5ea020f8ed611f13d994dad361af9, Height: 1922554
	*/

	// Chain Info
	let chain_info = client.rpc().chain_info().await?;
	println!("Best Hash: {:?}, Height: {}", chain_info.best_hash, chain_info.best_height);
	println!("Finalized Hash: {:?}, Height: {}", chain_info.finalized_hash, chain_info.finalized_height);
	println!("Genesis Hash: {:?}", chain_info.genesis_hash);
	/*
		Best 	  Hash: 0x1f1c30e3327487c5f0f2562fd4d6023f3582d30b3cd4f26e23304f845437732a, Height: 1922556
		Finalized Hash: 0xb5e82270ce67e77be50521768f8ebb6db7c5ea020f8ed611f13d994dad361af9, Height: 1922554
		Genesis   Hash: 0xb91746b45e0346cc2f815a520b9c6cb4d5c0902af848db0a80f85932d2e8276a
	*/

	// Block State
	let block_state = client.rpc().block_state(1900000).await?;
	match block_state {
		BlockState::Included => println!("Block Not Yet Finalized"),
		BlockState::Finalized => println!("Block Finalized"),
		BlockState::Discarded => println!("Block Discarded"),
		BlockState::DoesNotExist => println!("Block Does not Exist"),
	};
	/*
		Block Finalized
	*/

	// Block Header
	let at = Some(1900000);
	let best = client.best().block_header().await?;
	let finalized = client.finalized().block_header().await?;
	let specific = client.rpc().block_header(at).await?.expect("Should be there");
	println!("Best Header: Hash: {:?}, Height: {}", best.hash(), best.number);
	println!("Finalized Header: Hash: {:?}, Height: {}", finalized.hash(), finalized.number);
	println!("Specific Header: Hash: {:?}, Height: {}", specific.hash(), specific.number);
	/*
		Best Header: 	  Hash: 0x1f1c30e3327487c5f0f2562fd4d6023f3582d30b3cd4f26e23304f845437732a, Height: 1922556
		Finalized Header: Hash: 0xb5e82270ce67e77be50521768f8ebb6db7c5ea020f8ed611f13d994dad361af9, Height: 1922554
		Specific Header:  Hash: 0x6352f7d9cc7ac541170e769b8b7c0f0bcb22ebd6ba80bcd1313e265c33c911c7, Height: 1900000
	*/

	// Account Nonces
	let address = "5Ev16A8iWsEBFgtAxcyS8T5nDx8rZxWkg2ZywPgjup3ACSUZ";
	let best = client.best().account_nonce(address).await?;
	let finalized = client.finalized().account_nonce(address).await?;
	let specific = client.rpc().block_nonce(address, 1000000).await?;
	// RPC nonce is the one that you want 99.99% of time
	let rpc = client.rpc().account_nonce(address).await?;
	println!("Best Nonce: {}, Finalized Nonce: {}, Specific Nonce: {},", best, finalized, specific);
	println!("RPC Nonce: {}", rpc);
	/*
		Best Nonce: 	 26,
		Finalized Nonce: 26,
		Specific Nonce:  16,
		RPC Nonce: 		 26
	*/

	// Account Balances
	let address = "5FjdibsxmNFas5HWcT2i1AXbpfgiNfWqezzo88H2tskxWdt2";
	let best = client.best().account_balance(address).await?;
	let finalized = client.finalized().account_balance(address).await?;
	let specific = client.rpc().account_balance(address, 1000000).await?;
	println!(
		"Best Free Balance: {}, Finalized Free Balance: {}, Specific Free Balance: {}",
		best.free, finalized.free, specific.free
	);
	/*
		Best	  Free Balance: 219854606169762087553642,
		Finalized Free Balance: 219854606169762087553642,
		Specific  Free Balance: 66061836881671961431378
	*/

	// Account Info
	let address = "5GReLENC89bZfEQdytoMDY2krPnX1YC3qe14Gj3zFbjov4hX";
	let best = client.best().account_info(address).await?;
	let finalized = client.finalized().account_info(address).await?;
	let specific = client.rpc().account_info(address, 1000000).await?;
	println!("Best: Nonce: {},  Free Balance: {}", best.nonce, best.data.free);
	println!("Finalized: Nonce: {},  Free Balance: {}", finalized.nonce, finalized.data.free);
	println!("Specific: Nonce: {},  Free Balance: {}", specific.nonce, specific.data.free);
	/*
		Best: 	   Nonce: 149, Free Balance: 85596833424026384475153
		Finalized: Nonce: 149, Free Balance: 85596833424026384475153
		Specific:  Nonce: 7,   Free Balance: 50309645927248629897192
	*/

	Ok(())
}
