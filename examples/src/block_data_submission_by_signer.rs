use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;
	let block_hash = new_h256_from_hex("0x94746ba186876d7407ee618d10cb6619befc59eeb173cacb00c14d1ff492fc58")?;

	let block = Block::new(&sdk.client, block_hash).await?;

	// All Block Blobs by Signer
	let account_id = account_id_from_str("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();
	let blobs = block.data_submissions(Filter::new().tx_signer(account_id.clone()));
	assert_eq!(blobs.len(), 1, "Blobs must present 1 time");

	// Printout All Block Blobs by Signer
	for blob in blobs {
		let blob_data = blob.to_ascii().unwrap();
		assert_eq!(blob.account_id(), Some(account_id.clone()), "Signer must be the same.");

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
