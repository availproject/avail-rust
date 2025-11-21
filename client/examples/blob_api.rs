use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let blob_hash = H256::from_str("0x66fae1a211a6179044a9f85c2c16cbf747cdd2c19143880c690de4006768efb7").unwrap();
	let block_hash = H256::from_str("0x9139401fe68807814ea852d97e646a022ad07885b03a3a99ecfbb99735435824").unwrap();

	//
	// getBlob
	//
	// Mode 1: using a specific block
	let blob_from_block = client.chain().blob_get_blob(blob_hash, Some(block_hash)).await?;
	println!("getBlob (block): {:?}", blob_from_block);

	// Mode 2: using indexed storage info
	let blob_canonical = client.chain().blob_get_blob(blob_hash, None).await?;
	println!("getBlob (indexed_info): {:?}", blob_canonical);

	//
	// getBlobInfo
	//
	let info = client.chain().blob_get_blob_info(blob_hash).await?;
	println!("blobInfo: {:?}", info);

	//
	// inclusionProof
	//
	// Using Indexed info
	let proof = client.chain().blob_inclusion_proof(blob_hash, None).await?;
	println!("inclusion proof (indexed_info): {:?}", proof);

	// Using a specific block
	let proof_block = client.chain().blob_inclusion_proof(blob_hash, Some(block_hash)).await?;
	println!("inclusion proof(block): {:?}", proof_block);

	Ok(())
}
