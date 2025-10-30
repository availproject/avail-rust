use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Account Info
	let charlie = "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y";
	let best_account_info = client.best().account_info(charlie).await?;
	let finalized_account_info = client.finalized().account_info(charlie).await?;
	let account_info = client.chain().account_info(charlie, 2000000).await?;
	println!(
		"Best Block Charlie Nonce: {}, Free Balance: {}",
		best_account_info.nonce, best_account_info.data.free
	);
	println!(
		"Finalized Block Charlie Nonce: {}, Free Balance: {}",
		finalized_account_info.nonce, finalized_account_info.data.free
	);
	println!("Block 2000000 Charlie Nonce: {}, Free Balance: {}", account_info.nonce, account_info.data.free);

	Ok(())
}

/*
	Expected Output:

	Best Block Charlie Nonce: 299, Free Balance: 91772963578991329207
	Finalized Block Charlie Nonce: 299, Free Balance: 91772963578991329207
	Block 2000000 Charlie Nonce: 294, Free Balance: 92395139049599405067
*/
