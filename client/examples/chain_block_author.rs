use avail_rust_client::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Block Author
	let block_author = client.chain().block_author(2000000).await?;
	println!("Block 2000000 Author: {}", block_author);

	Ok(())
}

/*
	Expected Output:

	Block 2000000 Author: 5GQjARS9nVu5t3NrBZGhdUKKpwdj1xvnek9rNk7UKaH7DHoJ
*/
