use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Runtime API Raw Call
	let account_id = AccountId::from_str("5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y").expect("Should be Ok");
	let account_id = account_id.0;

	let nonce = client
		.chain()
		.runtime_api_raw_call::<u32>("AccountNonceApi_account_nonce", account_id.as_slice(), None)
		.await?;
	println!("AccountNonceApi_account_nonce: Charlie Nonce: {}", nonce);

	Ok(())
}

/*
	Expected Output:

	AccountNonceApi_account_nonce: Charlie Nonce: 299
*/
