use avail_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
	// Establishing a connection
	let client = Client::new(LOCAL_ENDPOINT).await?;

	// Defining account that will sign future transaction
	// let signer = Keypair::from_str("bottom drive obey lake curtain smoke basket hold race lonely fit walk")?;
	let signer = alice();
	// Or use one of dev accounts -> let signer = alice();

	// Transaction Creation
	let submittable_tx = client.tx().data_availability().submit_data(2, "My First Data Submission");

	// Transaction Submission
	let submitted_tx = submittable_tx.sign_and_submit(&signer, Options::default()).await?;
	println!("Tx Hash: {:?}", submitted_tx.tx_hash);
	/*
		Tx Hash: 0x4becd69e0730fa651ec08d2a444c92a50d2b920f1292cd478a11c39a8aedf626
	*/

	// Transaction Receipt
	let receipt = submitted_tx.receipt(false).await?;
	let Some(receipt) = receipt else {
		panic!("Oops, looks like our transaction was dropped")
	};
	println!("Block Hash: {:?}, Block Height: {}", receipt.block_ref.hash, receipt.block_ref.height);
	println!("Tx Hash: {:?}, Tx Index: {}", receipt.tx_ref.hash, receipt.tx_ref.index);
	/*
		Block Hash: 0x0f3fa78ee44a2626588976b99c685e3a83f326ebb2bbfacd357f24b4d83146ea, Block Height: 2333353
		Tx Hash: 0x4becd69e0730fa651ec08d2a444c92a50d2b920f1292cd478a11c39a8aedf626, Tx Index: 1
	*/

	let block_state = receipt.block_state().await?;
	match block_state {
		BlockState::Included => println!("Block Not Yet Finalized"),
		BlockState::Finalized => println!("Block Finalized"),
		BlockState::Discarded => println!("Block Discarded"),
		BlockState::DoesNotExist => println!("Block Does not Exist"),
	};
	/*
		Block Finalized
	*/

	Ok(())
}
