use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	let client = Client::new(TURING_ENDPOINT).await?;
	let submittable = client.tx().data_availability().submit_data(vec![0]);

	let estimated_fees = submittable.estimate_call_fees(None).await?;
	println!("Fees: {}", estimated_fees.final_fee());

	let estimated_fees = submittable
		.estimate_extrinsic_fees(&alice(), Options::new(2), None)
		.await?;
	println!("Fees: {}", estimated_fees.final_fee());

	Ok(())
}
