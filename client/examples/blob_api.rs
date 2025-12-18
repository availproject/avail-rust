use avail_rust_client::{avail_rust_core::rpc::blob::submit_blob, ext::const_hex, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	// For testing blob submission tx
	let blob = const_hex::decode("4141414141414141414141414141414141414141414141414141414141414141").unwrap();
	let blob_hash = H256::from_str("59cad5948673622c1d64e2322488bf01619f7ff45789741b15a9f782ce9290a8").unwrap();
	let commitments = const_hex::decode(
		"8adc43b724bedae8b4b593b9e10f2b251ef435a02c119100d5d81297e9c8fe1774a4e81d9e21ba50bd402461fd9080d0",
	)
	.unwrap();

	let signer = alice();
	let unsigned_tx =
		client
			.tx()
			.data_availability()
			.submit_blob_metadata(2, blob_hash, blob.len() as u64, commitments, None, None);

	let tx = unsigned_tx.sign(&signer, Options::default()).await.unwrap().encode();

	if let Err(e) = submit_blob(&client.rpc_client, &tx, &blob).await {
		println!("An error has occured: {e}");
	} else {
		println!("Blob submitted");
	}

	//  For testing blob RPCs
	// let blob_hash = H256::from_slice(&hex::decode("59cad5948673622c1d64e2322488bf01619f7ff45789741b15a9f782ce9290a8").unwrap());
	// let block_hash = H256::from_slice(&hex::decode("9139401fe68807814ea852d97e646a022ad07885b03a3a99ecfbb99735435824").unwrap());

	//
	// getBlob
	//
	// Mode 1: using a specific block
	// let blob_from_block = client
	//     .chain()
	//     .blob_get_blob(blob_hash, Some(block_hash))
	//     .await?;
	// println!("getBlob (block): {:?}", blob_from_block);

	// // Mode 2: using indexed storage info
	// let blob_canonical = client
	//     .chain()
	//     .blob_get_blob(blob_hash, None)
	//     .await?;
	// println!("getBlob (indexed_info): {:?}", blob_canonical);

	// //
	// // getBlobInfo
	// //
	// let info = client.chain().blob_get_blob_info(blob_hash).await?;
	// println!("blobInfo: {:?}", info);

	// //
	// // inclusionProof
	// //
	// // Using Indexed info
	// let proof = client
	//     .chain()
	//     .blob_inclusion_proof(blob_hash, None)
	//     .await?;
	// println!("inclusion proof (indexed_info): {:?}", proof);

	// // Using a specific block
	// let proof_block = client
	//     .chain()
	//     .blob_inclusion_proof(blob_hash, Some(block_hash))
	//     .await?;
	// println!("inclusion proof(block): {:?}", proof_block);

	Ok(())
}
