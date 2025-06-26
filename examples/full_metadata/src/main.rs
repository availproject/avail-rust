use avail_generated::runtime_types::bounded_collections::bounded_vec::BoundedVec;
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let data = BoundedVec(vec![0, 1, 2]);
	let payload = avail_generated::tx().data_availability().submit_data(data);
	let options = Options::new().app_id(2);

	// Payload can be converted to SubmittableTransaction
	let st = payload.to_submittable_transaction(client.clone())?;
	let st = st.sign_and_submit(&alice(), options).await?;
	let receipt = st.receipt(false).await?.expect("");
	println!(
		"Block Hash: {:?}, Block Height: {:?}",
		receipt.block_id.hash, receipt.block_id.height
	);

	// Payload can be converted to Transaction Call
	let call = payload.to_transaction_call(&client)?;
	let tx = SubmittableTransaction::new(client, call);
	let st = tx.sign_and_submit(&alice(), options).await?;
	let receipt = st.receipt(false).await?.expect("");
	println!(
		"Block Hash: {:?}, Block Height: {:?}",
		receipt.block_id.hash, receipt.block_id.height
	);

	Ok(())
}
