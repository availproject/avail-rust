use avail_rust::{
	avail::{
		RuntimeCall,
		utility::{
			events::{
				BatchCompleted, BatchCompletedWithErrors, BatchInterrupted, DispatchedAs, ItemCompleted, ItemFailed,
			},
			tx::BatchAll,
		},
	},
	prelude::*,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let balances = client.tx().balances();
	let c1 = balances.transfer_keep_alive(bob().account_id(), constants::ONE_AVAIL);
	let c2 = balances.transfer_keep_alive(dave().account_id(), constants::ONE_AVAIL);

	// There are three batch calls:
	// 1. Batch, 2. Batch All and 3. Force Batch
	let tx = client.tx().utility().batch_all(vec![c1, c2]);
	let st = tx.sign_and_submit(&alice(), Options::default()).await?;
	let Some(receipt) = st.receipt(false).await? else {
		return Err("Transaction got dropped. This should never happen in a local network.".into());
	};

	// Fetching and displaying Transaction Events
	let events = receipt.events().await?;
	assert!(events.is_extrinsic_success_present());

	// Batch, Batch All and Force Batch can emit different events.
	if events.is_present::<BatchInterrupted>() {
		println!("Found Utility::BatchInterrupted");
	}
	if events.is_present::<BatchCompleted>() {
		println!("Found Utility::BatchCompleted");
	}
	if events.is_present::<BatchCompletedWithErrors>() {
		println!("Found Utility::BatchCompletedWithErrors");
	}

	println!("Found {}x Utility::ItemCompleted", events.count::<ItemCompleted>());

	if events.is_present::<ItemFailed>() {
		println!("Found Utility::ItemFailed");
	}
	if events.is_present::<DispatchedAs>() {
		println!("Found Utility::DispatchedAs");
	}
	/*
		Found Utility::BatchCompleted
		Found 2x Utility::ItemCompleted
	*/

	// Decoding batch call
	let tx = receipt.tx::<BatchAll>().await?;
	let decoded_calls = tx.call.decode_calls()?;
	for call in decoded_calls {
		// RuntimeCall variants are the only calls that are decodable from a batch call.
		// If the call is not a RuntimeCall variant then it won't be decodable by the
		// Batch call
		let RuntimeCall::BalancesTransferKeepAlive(tx) = call else {
			return Err("Expected Balance Transfer Keep Alive".into());
		};

		// If MultiAddress is of variant ID then map it to ss58 address otherwise
		// display the debug information
		let dest = AccountId::try_from(&tx.dest)
			.map(|x| x.to_string())
			.unwrap_or_else(|_| std::format!("{:?}", tx.dest));

		println!("Dest: {:?}, Amount: {}", dest, tx.value);
	}
	/*
		Dest: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", Amount: 1000000000000000000
		Dest: "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", Amount: 1000000000000000000
	*/

	Ok(())
}
