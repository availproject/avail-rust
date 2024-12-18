use avail_rust::error::ClientError;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	// RPC Connection
	// ANCHOR: connection
	use avail_rust::{
		subxt::backend::rpc::{
			reconnecting_rpc_client::RpcClient as ReconnectingRpcClient, RpcClient,
		},
		AOnlineClient,
	};

	let endpoint = "ws://127.0.0.1:9944";
	let rpc_client = ReconnectingRpcClient::builder().build(endpoint).await;
	let rpc_client = rpc_client.map_err(|e| e.to_string())?;

	let rpc_client = RpcClient::new(rpc_client);
	let online_client = AOnlineClient::from_rpc_client(rpc_client.clone()).await?;
	// ANCHOR_END: connection

	// Accounts
	// ANCHOR: accounts
	use avail_rust::subxt_signer::{sr25519::Keypair, SecretUri};
	use std::str::FromStr;

	let secret_uri = SecretUri::from_str("//Alice")?;
	let account = Keypair::from_uri(&secret_uri)?;
	let account_id = account.public_key().to_account_id();
	// 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
	let _account_address = account_id.to_string();
	// ANCHOR_END: accounts

	// Payload
	// ANCHOR: payload
	use avail_rust::{
		avail::runtime_types::bounded_collections::bounded_vec::BoundedVec,
		subxt::{blocks::StaticExtrinsic, ext::subxt_core::tx::payload::StaticPayload},
	};

	use avail_rust::avail::data_availability::calls::types::SubmitData;
	let pallet_name = SubmitData::PALLET;
	let call_name = SubmitData::CALL;

	let data = String::from("My Data").into_bytes();
	let data = BoundedVec(data);
	let call_data = SubmitData { data };

	let payload = StaticPayload::new(pallet_name, call_name, call_data);
	// ANCHOR_END: payload

	// Transaction Parameters
	// ANCHOR: params
	use avail_rust::Options;
	let options = Options::new()
		.build(&online_client, &rpc_client, &account_id)
		.await?;
	let params = options.build().await?;
	// ANCHOR_END: params

	// Signature
	// ANCHOR: signature
	let submittable_tx = online_client
		.tx()
		.create_signed(&payload, &account, params)
		.await?;
	// ANCHOR_END: signature

	// Submission
	// ANCHOR: submission
	let tx_hash = submittable_tx.submit().await?;
	// ANCHOR_END: submission

	// Watcher
	// ANCHOR: watcher
	use avail_rust::avail::system::events::ExtrinsicSuccess;
	let mut block_sub = online_client.blocks().subscribe_all().await?;
	while let Some(block) = block_sub.next().await {
		let block = block?;
		let block_txs = block.extrinsics().await?;
		let tx = block_txs.iter().find(|tx| tx.hash() == tx_hash);
		if let Some(tx) = tx {
			println!("Transaction was found.");
			println!("Block Hash: {:?}", block.hash()); // Block Hash: 0x61415b6012005665bac0cf8575a94e509d079a762be2ba6a71a04633efd01c1b
			println!("Block Number: {:?}", block.number()); // Block Number: 200
			println!("Tx Hash: {:?}", tx.hash()); // Tx Hash: 0x01651a93d55bde0f258504498d4f2164416df5331794d9c905d4c8711d9537ef
			println!("Tx Index: {:?}", tx.index()); // Tx Index: 1

			let events = tx.events().await?;
			println!("Event count: {}", events.iter().count()); // Event count: 7
			if events
				.find_first::<ExtrinsicSuccess>()
				.ok()
				.flatten()
				.is_some()
			{
				println!("Transaction was successful");
			}

			break;
		}
	}
	// ANCHOR_END: watcher

	Ok(())
}
