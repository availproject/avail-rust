use avail::RuntimeCall;
use avail_rust::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_logging();
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let c = client.clone();
	let t1 = tokio::spawn(async move { transaction_pushing(c).await });
	let c = client.clone();
	let t2 = tokio::spawn(async move { block_indexing(c).await });

	t1.await.unwrap()?;
	wait_for_new_block(&client).await?;
	t2.abort();

	Ok(())
}

async fn transaction_pushing(client: Client) -> Result<(), ClientError> {
	let message = String::from("It works").as_bytes().to_vec();
	let tx = client.tx().data_availability().submit_data(message);
	let count = 5;
	for i in 0..count {
		tx.sign_and_submit(&alice(), Options::new()).await?;
		println!("Transaction submitted. {} done out of {}", i + 1, count);
		if i < (count - 1) {
			wait_for_new_block(&client).await?;
		}
	}

	Ok(())
}

async fn block_indexing(client: Client) -> Result<(), ClientError> {
	use avail::data_availability::tx::Call;

	let mut next_height = client.best_block_height().await?;
	loop {
		let current_height = client.best_block_height().await?;
		if next_height > current_height {
			tokio::time::sleep(Duration::from_secs(5)).await;
			continue;
		}

		let Some(hash) = client.block_hash(next_height).await? else {
			next_height += 1;
			continue;
		};

		let Some(block) = client.block(hash).await? else {
			next_height += 1;
			continue;
		};

		for enc_tx in &block.block.extrinsics {
			let Ok(tx) = OpaqueTransaction::try_from(enc_tx) else {
				continue;
			};

			let Ok(runtime_call) = RuntimeCall::try_from(&tx.call) else {
				continue;
			};

			let RuntimeCall::DataAvailability(Call::SubmitData(sd)) = runtime_call else {
				continue;
			};

			println!(
				"Found Submit Data transaction. Data: {:?}. Block Height: {}",
				sd.data, next_height
			);
		}

		next_height += 1;
	}
}

async fn wait_for_new_block(client: &Client) -> Result<(), ClientError> {
	let next_height = client.best_block_height().await? + 1;
	loop {
		let current_height = client.best_block_height().await?;
		if next_height > current_height {
			tokio::time::sleep(Duration::from_secs(5)).await;
			continue;
		}

		return Ok(());
	}
}
