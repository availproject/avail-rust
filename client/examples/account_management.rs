use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	// Creating Keypair
	let seed = "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Charlie";
	let my_secret_uri = SecretUri::from_str(seed).expect("Should work");
	let my_signer = Keypair::from_uri(&my_secret_uri).expect("Should work");
	println!("Account SS58 Address: {}", my_signer.public_key().to_account_id());

	// Shortcut
	let my_signer = Keypair::from_str(seed).expect("Should work");
	println!("Account SS58 Address: {}", my_signer.public_key().to_account_id());

	// Access to dev accounts
	let _alice_signer = alice();
	let _bob_signer = bob();
	let _charlie_signer = charlie();
	let _eve_signer = eve();

	// Nonce & Balance
	let client = Client::new(TURING_ENDPOINT).await?;
	let account_id = my_signer.public_key().to_account_id();
	let nonce = client.chain().account_nonce(account_id.clone()).await?;
	let balance = client.best().account_balance(account_id.clone()).await?;
	println!("Charlie Nonce: {}, Free Balance: {}", nonce, balance.free);

	// Historical Nonce & Balance
	let account_info = client.chain().account_info(account_id, 2000000).await?;
	println!("Charlie (block 2000000) Nonce: {}, Free Balance: {}", account_info.nonce, account_info.data.free);

	Ok(())
}

/*
	Expected Output:

	Account SS58 Address: 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y
	Account SS58 Address: 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y
	Charlie Nonce: 297, Free Balance: 92021833748514619844
	Charlie (block 2000000) Nonce: 294, Free Balance: 92395139049599405067
*/
