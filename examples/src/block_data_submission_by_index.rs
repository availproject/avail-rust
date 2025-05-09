use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;
	let block_hash = new_h256_from_hex("0x94746ba186876d7407ee618d10cb6619befc59eeb173cacb00c14d1ff492fc58")?;

	let block = Block::new(&sdk.client, block_hash).await?;

	// All Block Blobs by Index
	let tx_index = 6;
	let blobs = block.data_submissions(Filter::new().tx_index(tx_index));
	assert_eq!(blobs.len(), 1, "");

	let blob = &blobs[0];

	// Printout All Block Blobs by Index
	let blob_data = blob.to_ascii().unwrap();
	assert_eq!(blob.tx_index, tx_index, "Tx Index must be the same");

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
