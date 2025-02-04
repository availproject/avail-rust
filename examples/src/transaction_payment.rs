use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::turing_endpoint()).await?;
	let account = account::alice();

	let key = String::from("My Key").into_bytes();
	let tx = sdk.tx.data_availability.create_application_key(key);
	let options = Options::new();

	let fee_details = tx.payment_query_fee_details(&account, Some(options)).await?;
	let inclusion_fee = fee_details.inclusion_fee.unwrap();
	println!(
		"Adjusted Weight Fee: {}, Len Fee: {}, Base Fee: {}",
		inclusion_fee.adjusted_weight_fee, inclusion_fee.len_fee, inclusion_fee.base_fee
	);

	let fee_details = tx.payment_query_call_fee_details().await?;
	let inclusion_fee = fee_details.inclusion_fee.unwrap();
	println!(
		"Adjusted Weight Fee: {}, Len Fee: {}, Base Fee: {}",
		inclusion_fee.adjusted_weight_fee, inclusion_fee.len_fee, inclusion_fee.base_fee
	);

	let info = tx.payment_query_info(&account, Some(options)).await?;
	println!(
		"ProofSize: {}, RefTime: {}, Class: {:?}, Partial Fee: {}",
		info.weight.proof_size, info.weight.ref_time, info.class, info.partial_fee
	);

	let info = tx.payment_query_call_info().await?;
	println!(
		"ProofSize: {}, RefTime: {}, Class: {:?}, Partial Fee: {}",
		info.weight.proof_size, info.weight.ref_time, info.class, info.partial_fee
	);

	println!("Transaction Options finished correctly");

	Ok(())
}
