use avail_rust_client::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	let key = std::format!("{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
	let submittable = client.tx().data_availability().create_application_key(key);
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
		.first::<avail::data_availability::events::ApplicationKeyCreated>()
		.expect("Should be there");
	println!(
		"Is Successful: {}, Id: {}, key: {}, Owner: {}",
		events.is_extrinsic_success_present(),
		event.id,
		String::from_utf8(event.key).unwrap(),
		event.owner
	);

	// Fetching Extrinsic itself
	let ext = receipt
		.extrinsic::<avail::data_availability::tx::CreateApplicationKey>()
		.await?;
	println!("Key: {}", String::from_utf8(ext.call.key).unwrap());

	Ok(())
}

/*
	Expected Output:

	Ext Hash: 0x49256fb54bc13df2da57262d9b7162bd7fe36d2368721058a3b2ee67d4862c8e
	Block State: Finalized
	Block Height: 2495977, Block Hash: 0x2e068ec7cdee82dc27d0917648b33da2e64174c2f38c0138cbc0d35530eca90e, Ext Hash: 0x49256fb54bc13df2da57262d9b7162bd7fe36d2368721058a3b2ee67d4862c8e, Ext Index: 2
	Is Successful: true, Id: 496, key: 1761661019, Owner: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
	Key: 1761661019
*/
