use avail_rust_client::prelude::*;
use avail_rust_client::subscription::Subscriber;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(TURING_ENDPOINT).await?;

	let mut header_sub = client.subscription_justifications(Subscriber::new_best_block(1000, 0));
	let mut i = 0;
	while let Ok((sub, block)) = header_sub.next().await {
		println!("Found justification on block {}", block.0);

		i += 1;
		if i > 3 {
			break;
		}
	}

	Ok(())
}
