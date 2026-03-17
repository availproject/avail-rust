use avail_fri::B128;
use avail_fri::BytesEncoder;
use avail_fri::FriBiniusPCS;
use avail_fri::FriParamsVersion;
use avail_fri::eval_utils::derive_evaluation_point;
use avail_fri::eval_utils::derive_seed_from_inputs;
use avail_fri::eval_utils::eval_claim_to_bytes;
use avail_rust_client::ext::sp_crypto_hashing::keccak_256;
use avail_rust_client::prelude::*;
use avail_rust_core::avail::babe::storage::BabeRandomness;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let client = Client::connect("").await?;

	// Data
	let data = "My Data".as_bytes();
	let data_hash = H256::from(keccak_256(&data));
	let randomness = BabeRandomness::fetch(&client.rpc_client, None).await?.unwrap();
	let (commitments, seed, claim) = compute_extra_data(&randomness, &data, &data_hash.0)?;

	let result = client
		.blob()
		.submit_blob_and_blob_metadata(2, &data, data_hash, commitments, seed, claim, &alice(), Options::new())
		.await;

	println!("{:?}", data_hash);

	if let Err(e) = result {
		println!("An error has occured: {e}");
	} else {
		println!("Blob submitted");
	}

	// let hash = H256::from_str("0x47a59a7805e0bfe350ee0395d426c15770edc03fee72aa6532b5bbcffaf28030").unwrap();
	// let info = client.blob().info(hash).await?;

	Ok(())
}

fn compute_extra_data(
	randomness: &[u8; 32],
	data: &[u8],
	data_hash: &[u8; 32],
) -> anyhow::Result<(Vec<u8>, Option<[u8; 32]>, Option<[u8; 16]>)> {
	let encoder = BytesEncoder::<B128>::new();
	let packed = encoder.bytes_to_packed_mle(data).unwrap();
	let cfg = FriParamsVersion::V0.to_config(packed.total_n_vars);
	let pcs = FriBiniusPCS::new(cfg);
	let ctx = pcs.initialize_fri_context::<B128>(packed.packed_mle.log_len()).unwrap();

	let commit_output = pcs.commit(&packed.packed_mle, &ctx).unwrap();

	let seed = derive_seed_from_inputs(randomness, data_hash);
	let evaluation_point = derive_evaluation_point(seed, packed.total_n_vars);
	let eval_claim = pcs
		.calculate_evaluation_claim(&packed.packed_values, &evaluation_point)
		.unwrap();

	let claim: [u8; 16] = eval_claim_to_bytes(eval_claim).try_into()?;
	let commitment = commit_output.commitment.to_vec();
	Ok((commitment, Some(seed), Some(claim)))
}
