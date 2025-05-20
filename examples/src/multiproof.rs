use avail_rust::{avail::runtime_types::avail_core::header::extension::HeaderExtension, prelude::*, rpc::chain, Cell};
use kate_recovery::{data::GCellBlock, matrix::Dimensions};

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

	let (rows, cols) = match header.unwrap().extension {
		HeaderExtension::V3(ext) => (ext.commitment.rows, ext.commitment.cols),
	};

	let target_dims = Dimensions::new_from(16, 64).unwrap();

	let final_dims =
		avail_rust::utils::generate_multiproof_grid_dims(Dimensions::new_from(rows, cols).unwrap(), target_dims)
			.unwrap();

	let mut cells = Vec::new();
	for row in 0..final_dims.rows().get() {
		for col in 0..final_dims.cols().get() {
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

	let pmp = avail_rust::rpc::kate::generate_pmp().await;

	println!("Starting verify_multi_proof...");
	let start_time = std::time::Instant::now();

	let proofs_bytes: Vec<((Vec<[u8; 32]>, [u8; 48]), GCellBlock)> = proofs
		.iter()
		.map(|((values, proof), block)| {
			let values_bytes: Vec<[u8; 32]> = values.iter().map(|v| v.to_big_endian()).collect();
			((values_bytes, proof.0), block.clone())
		})
		.collect();

	let verify = match kate_recovery::proof::verify_multi_proof(&pmp, &proofs_bytes, &commitments, cols as usize).await
	{
		Ok(v) => v,
		Err(e) => {
			println!("Error: {:?}", e);
			false
		},
	};
	let verify_duration = start_time.elapsed();
	println!("verify_multi_proof completed in {:?}", verify_duration);

	assert!(verify, "Proof verification failed");
	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	run().await
}
