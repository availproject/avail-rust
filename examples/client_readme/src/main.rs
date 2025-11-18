use avail_rust_client::prelude::*;

#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomExtrinsic {
	pub data: Vec<u8>,
}
impl HasHeader for CustomExtrinsic {
	const HEADER_INDEX: (u8, u8) = (29u8, 1u8);
}

#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomEvent {
	pub who: AccountId,
	pub data_hash: H256,
}
impl HasHeader for CustomEvent {
	const HEADER_INDEX: (u8, u8) = (29, 1);
}

pub struct DataAvailabilityAppKeys;
impl StorageMap for DataAvailabilityAppKeys {
	type KEY = Vec<u8>;
	type VALUE = AppKey;

	const KEY_HASHER: StorageHasher = StorageHasher::Blake2_128Concat;
	const PALLET_NAME: &str = "DataAvailability";
	const STORAGE_NAME: &str = "AppKeys";
}
#[derive(Debug, Clone, codec::Decode)]
pub struct AppKey {
	pub owner: AccountId,
	#[codec(compact)]
	pub id: u32,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	// 1
	// Establishing a connection
	let client = Client::new(TURING_ENDPOINT).await?;

	// Defining account that will sign future transaction
	let signer = Keypair::from_str("bottom drive obey lake curtain smoke basket hold race lonely fit walk")?;
	// Or use one of dev accounts -> let signer = alice();

	// Transaction Creation
	let submittable_tx = client.tx().data_availability().submit_data(AppId(2), "My First Data Submission");

	// Transaction Submission
	let submitted_tx = submittable_tx.sign_and_submit(&signer, Options::default()).await?;
	println!("Tx Hash: {:?}", submitted_tx.tx_hash);

	// Transaction Receipt
	let receipt = submitted_tx.receipt(false).await?;
	let Some(receipt) = receipt else {
		panic!("Oops, looks like our transaction was dropped")
	};
	println!("Block Hash: {:?}, Block Height: {}", receipt.block_ref.hash, receipt.block_ref.height);
	println!("Tx Hash: {:?}, Tx Index: {}", receipt.tx_ref.hash, receipt.tx_ref.index);

	let block_state = receipt.block_state().await?;
	match block_state {
		BlockState::Included => println!("Block Not Yet Finalized"),
		BlockState::Finalized => println!("Block Finalized"),
		BlockState::Discarded => println!("Block Discarded"),
		BlockState::DoesNotExist => println!("Block Does not Exist"),
	};

	// 2
	let client = Client::new(TURING_ENDPOINT).await?;
	let custom_call = CustomExtrinsic { data: vec![0, 1, 2, 3] };
	let submittable = SubmittableTransaction::new(client.clone(), ExtrinsicCall::from(&custom_call));
	let submitted = submittable.sign_and_submit(&alice(), Options::new(2)).await?;
	let receipt = submitted.receipt(true).await?.expect("Must be there");
	println!("Block Hash: {:?}", receipt.block_ref.hash);

	// 3
	let encoded_event = vec![0, 1, 2, 3];
	let _event = CustomEvent::from_event(&encoded_event);
	//println!("Account: {}, Hash: {}", event.who, event.data_hash);

	// 4
	let client = Client::new(TURING_ENDPOINT).await?;
	let block_hash = client.finalized().block_hash().await?;

	let key = "MyAwesomeKey".to_string().into_bytes();
	// Fetching Storage Map
	let value = DataAvailabilityAppKeys::fetch(&client.rpc_client, &key, Some(block_hash))
		.await?
		.expect("Needs to be there");
	println!("Owner: {}, id: {}", value.owner, value.id);

	// Iterating Storage Map
	let mut iter = DataAvailabilityAppKeys::iter(client.rpc_client.clone(), block_hash);
	for _ in 0..5 {
		let value = iter.next().await?.expect("Needs to be there");
		println!("Owner: {}, id: {}", value.owner, value.id);

		let (key, value) = iter.next_key_value().await?.expect("Needs to be there");
		println!("Key: {}, Owner: {}, id: {}", String::from_utf8(key).expect(""), value.owner, value.id);
	}

	Ok(())
}
