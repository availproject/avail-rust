use avail_rust::prelude::*;
use std::time::Duration;
use tokio::{task::JoinHandle, time::sleep};

pub async fn run() -> Result<(), ClientError> {
	let client = Client::new(LOCAL_ENDPOINT).await?;
	let mut futures: Vec<JoinHandle<Result<(), ClientError>>> = Vec::new();

	for signer in [alice(), bob(), charlie(), dave()] {
		let s = client.clone();
		let t = tokio::spawn(async move { task(s, signer).await });
		futures.push(t);
	}

	for fut in futures {
		fut.await.unwrap()?;
	}

	Ok(())
}

async fn task(client: Client, account: Keypair) -> Result<(), ClientError> {
	let message = String::from("It works").as_bytes().to_vec();
	let tx = client.tx().data_availability().submit_data(message);
	let st = tx.sign_and_submit(&account, Options::new()).await?;
	'outer: loop {
		let Some(receipt) = st.receipt(false).await? else {
			return Err("Transaction got dropped. This should never happen in a local network.".into());
		};

		loop {
			let block_state: BlockState = receipt.block_state().await?;
			match block_state {
				BlockState::Included => (),
				BlockState::Finalized => {
					return Ok(());
				},
				BlockState::Discarded => {
					break 'outer;
				},
				BlockState::DoesNotExist => {
					return Err("Block got dropped. This should never happen in a local network.".into());
				},
			};
			sleep(Duration::from_secs(5)).await;
		}
	}

	Ok(())
}
