use avail_fri::BlobCommitment;
use avail_rust_client::ext::sp_crypto_hashing::keccak_256;
use avail_rust_client::prelude::*;
use avail_rust_core::avail::babe::storage::BabeRandomness;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let client = Client::connect(LOCAL_ENDPOINT).await?;

	// Data
	let data = "My Data".as_bytes();
	let data_hash = H256::from(keccak_256(&data));
	let randomness = BabeRandomness::fetch(&client.rpc_client, None).await?.unwrap();
	let commitment = BlobCommitment::compute(&randomness, &data, &data_hash.0).unwrap();

	let result = client
		.blob()
		.submit_with_metadata_and_watch(
			2,
			&data,
			data_hash,
			commitment.commitment,
			Some(commitment.seed),
			Some(commitment.claim),
			&alice(),
			Options::new(),
			WaitOption::default(),
		)
		.await;

	println!("{:?}", result);

	if let Err(e) = result {
		println!("An error has occured: {e}");
	} else {
		println!("Blob submitted");
	}

	// let hash = H256::from_str("0x47a59a7805e0bfe350ee0395d426c15770edc03fee72aa6532b5bbcffaf28030").unwrap();
	// let info = client.blob().info(hash).await?;

	Ok(())
}
