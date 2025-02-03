use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	let sdk = SDK::new(SDK::local_endpoint()).await?;
	let account = SDK::alice()?;

	let key = String::from("My Key").into_bytes();
	let tx = sdk.tx.data_availability.create_application_key(key);

	let options = Options::new();
	let fee_details = tx.payment_query_fee_details(&account, Some(options)).await?;
	let query_info = tx.payment_query_info(&account, Some(options)).await?;

	dbg!(fee_details);
	dbg!(query_info);

	Ok(())
}

/*
	Example Output:

	fee_details = FeeDetails {
	inclusion_fee: Some(
		InclusionFee {
			base_fee: 124414000000000000,
			len_fee: 11900000000000,
			adjusted_weight_fee: 2743751768732346,
		},
	),
	tip: 0,

	query_info = 127169255884363086
}
*/
