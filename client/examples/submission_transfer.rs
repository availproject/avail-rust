use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let submittable = client
		.tx()
		.balances()
		.transfer_keep_alive("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ", ONE_AVAIL);
	let signer = Keypair::from_str("//Bob")?;
	let submitted = submittable.sign_and_submit(&signer, Default::default()).await?;
	println!("Ext Hash: {:?}", submitted.ext_hash,);

	// Getting Extrinsic Receipt
	let receipt = submitted.receipt(false).await?.expect("Should be included");
	println!("Block State: {}", receipt.block_state().await?);
	println!(
		"Block Height: {}, Block Hash: {:?}, Ext Hash: {:?}, Ext Index: {}",
		receipt.block_height, receipt.block_hash, receipt.ext_hash, receipt.ext_index
	);

	// Fetching Extrinsic Events
	let events = receipt.events().await?;
	let event = events
		.first::<avail::balances::events::Transfer>()
		.expect("Should be there");
	println!(
		"Is Successful: {}, Amount: {}, From: {}, To: {}",
		events.is_extrinsic_success_present(),
		event.amount,
		event.from,
		event.to,
	);

	// Fetching Extrinsic itself
	let ext = receipt.extrinsic::<avail::balances::tx::TransferKeepAlive>().await?;
	let dest = match &ext.call.dest {
		MultiAddress::Id(id) => std::format!("{}", id),
		_ => std::format!("{:?}", ext.call.dest),
	};
	println!("Dest: {}, Value: {}", dest, ext.call.value);

	Ok(())
}

/*
	Expected Output:

	Ext Hash: 0x87b16bf8080967c1211cb5902e1573384cf082d26db33cb43417d4df8eb42c7d
	Block State: Finalized
	Block Height: 2496039, Block Hash: 0x58dd8ccde5a7cc7a473c0209651dcde8d0269d33c80c43055815a7297eb19a22, Ext Hash: 0x87b16bf8080967c1211cb5902e1573384cf082d26db33cb43417d4df8eb42c7d, Ext Index: 1
	Is Successful: true, Amount: 1000000000000000000, From: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty, To: 5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ
	Dest: 5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ, Value: 1000000000000000000
*/
