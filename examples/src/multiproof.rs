use avail_rust::{avail::runtime_types::avail_core::header::extension::HeaderExtension, prelude::*, rpc::chain, Cell};
use kate_recovery::matrix::Dimensions;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;

	let account = account::alice();

	println!("Starting Data Submission...");
	let data = [99u8; 256].to_vec();
	let options = Options::new().app_id(2);
	let tx = sdk.tx.data_availability.submit_data(data);
	let res = tx.execute_and_watch_finalization(&account, options).await?;
	assert_eq!(res.is_successful(), Some(true), "Transactions must be successful");

	println!("Block Hash: {:?}, Block Number: {}", res.block_hash, res.block_number);

	let header = chain::get_header(&sdk.client, Some(res.block_hash.clone())).await?;

	let (_rows, cols) = match header.extension {
		HeaderExtension::V3(ext) => (ext.commitment.rows, ext.commitment.cols),
	};

	let target_dims = Dimensions::new_from(1, 8).unwrap();

	let mut cells = Vec::new();
	for row in 0..target_dims.rows().get() {
		for col in 0..target_dims.cols().get() {
			cells.push(Cell {
				row: row as u32,
				col: col as u32,
			});
		}
	}

	println!("Data Submission finished correctly");
	println!("Starting query_multi_proof...");
	let start_time = std::time::Instant::now();
	let (proofs, commitments) =
		avail_rust::rpc::kate::query_multi_proof(&sdk.client, Some(res.block_hash), cells).await?;
	let query_duration = start_time.elapsed();
	println!("query_multi_proof completed in {:?}", query_duration);

	println!("Starting verify_multi_proof...");
	let start_time = std::time::Instant::now();
	let verify = avail_rust::rpc::kate::verify_multi_proof(proofs, commitments, cols as usize)
		.await
		.unwrap();
	let verify_duration = start_time.elapsed();
	println!("verify_multi_proof completed in {:?}", verify_duration);

	assert!(verify, "Proof verification failed");
	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	run().await
}
