use std::str::FromStr;

use avail_rust::prelude::*;

pub async fn run() -> Result<(), ClientError> {
	// Use SecretUri and Keypair to create your own account...
	let secret_uri = SecretUri::from_str("//Alice")?;
	let acc = Keypair::from_uri(&secret_uri)?;
	println!("Alice Address: {}", acc.public_key().to_account_id());

	// ... or from_secret_uri function
	let acc = account::from_secret_uri("//Alice")?;
	println!("Alice Address: {}", acc.public_key().to_account_id());

	// There are predefined testing accounts available to be used on local dev networks.
	let acc = account::alice();
	println!("Alice Address: {}", acc.public_key().to_account_id());
	let acc = account::bob();
	println!("Bob Address: {}", acc.public_key().to_account_id());
	let acc = account::charlie();
	println!("Charlie Address: {}", acc.public_key().to_account_id());
	let acc = account::dave();
	println!("Dave Address: {}", acc.public_key().to_account_id());
	let acc = account::eve();
	println!("Eve Address: {}", acc.public_key().to_account_id());
	let acc = account::ferdie();
	println!("Ferdie Address: {}", acc.public_key().to_account_id());

	// AccountId can be created form Keypair...
	let account_id = acc.public_key().to_account_id();
	println!("Ferdie Address: {}", account_id);

	// ...or from SS58 address
	let account_id = account::account_id_from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")?;
	println!("Alice Address: {}", account_id);

	// SS58 address can be created from Account ID
	println!("Alice Address: {}", account_id);

	// MultiAddress can be created from Public Key...
	let _address = acc.public_key().to_address::<u32>();

	// ...or from account id
	let _address = MultiAddress::from(account_id);

	Ok(())
}
