use avail_rust::{prelude::*, utils::new_h256_from_hex};

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;
	let block_hash = new_h256_from_hex("0x94746ba186876d7407ee618d10cb6619befc59eeb173cacb00c14d1ff492fc58")?;

	let block = Block::new(&sdk.client, block_hash).await?;

	// All Block Blobs
	let blobs = block.data_submissions(Filter::default());
	assert_eq!(blobs.len(), 4, "Blobs must present 4 times");

	// Printout All Block Blobs
	for blob in blobs {
		let blob_data = blob.to_ascii().unwrap();

		println!(
			"Tx Hash: {:?}, Tx Index: {}, Data: {:?}, App Id: {}, Tx Singer: {:?}",
			blob.tx_hash,
			blob.tx_index,
			blob_data,
			blob.app_id,
			blob.ss58address(),
		);
	}

	Ok(())
}
