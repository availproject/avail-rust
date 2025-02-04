use avail_rust::{prelude::*, utils::new_h256_from_hex};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;
	let block_hash = new_h256_from_hex("0x94746ba186876d7407ee618d10cb6619befc59eeb173cacb00c14d1ff492fc58")?;

	let block = Block::new(&sdk.client, block_hash).await?;

	// All Block Blobs by Hash
	let tx_hash = new_h256_from_hex("0xe7efa71363d11bce370fe71a33e5ff296775f37507075c49316132131420f793")?;
	let blob = block.data_submissions_by_hash(tx_hash);
	assert!(blob.is_some(), "Blob must present");

	let blob = blob.unwrap();

	// Printout All Block Blobs by Hash
	let blob_data = blob.to_ascii().unwrap();
	assert_eq!(blob.tx_hash, tx_hash, "Tx Hash must be the same");

	println!(
		"Tx Hash: {:?}, Tx Index: {}, Data: {:?}, App Id: {}, Tx Singer: {:?}",
		blob.tx_hash,
		blob.tx_index,
		blob_data,
		blob.app_id,
		blob.ss58address(),
	);

	Ok(())
}
