use avail_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;
	let submittable = client.tx().data_availability().submit_data(AppId(2), vec![0]);

	// Estimates call fee
	let estimated_fees = submittable.estimate_call_fees(None).await?;
	println!("Fees: {}", estimated_fees.final_fee());

	// Estimates whole extrinsic fee
	let estimated_fees = submittable
		.estimate_extrinsic_fees(&alice(), Options::new(2), None)
		.await?;
	println!("Fees: {}", estimated_fees.final_fee());

	Ok(())
}
