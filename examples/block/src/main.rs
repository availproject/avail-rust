use avail::data_availability::events::Event;
use avail::data_availability::tx::Call;
use avail_rust::ext::subxt_core::config::Header;
use avail_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_logging();
	let client = Client::new(TURING_ENDPOINT).await?;
	let blocks = client.block_client();

	let hash = H256::from_str("0x8222d6961ac9c4fd7d10fad5e1fc42807a49aab158d2560482910ed211fc7592").expect("qed");
	let block = blocks.block(hash).await?.expect("Should be there");
	println!("Height: {}", block.header.number);

	println!("Transactions: OpaqueTransaction + RuntimeCall");
	for raw_ext in block.extrinsics.iter() {
		let Ok(opaque_tx) = OpaqueTransaction::try_from(raw_ext) else {
			continue;
		};

		println!(
			"Pallet index: {}, Call index: {}",
			opaque_tx.pallet_index(),
			opaque_tx.call_index()
		);

		let Ok(runtime_call) = RuntimeCall::try_from(&opaque_tx.call) else {
			continue;
		};

		let RuntimeCall::DataAvailability(Call::SubmitData(sd)) = &runtime_call else {
			continue;
		};

		let address = opaque_tx.signature.as_ref().map(|x| x.address.clone()).expect("qed");
		let MultiAddress::Id(account_id) = address else {
			continue;
		};

		println!("Address: {}, Submitted data: {:?}", account_id, &sd.data[0..3])
	}

	println!("\nTransactions: DecodedTransaction");
	for raw_ext in block.extrinsics.iter() {
		let Ok(decoded_tx) = DecodedTransaction::try_from(raw_ext) else {
			continue;
		};

		println!(
			"Pallet index: {}, Call index: {}",
			decoded_tx.pallet_index(),
			decoded_tx.call_index()
		);

		let RuntimeCall::DataAvailability(Call::SubmitData(sd)) = &decoded_tx.call else {
			continue;
		};

		let address = decoded_tx.signature.as_ref().map(|x| x.address.clone()).expect("qed");
		let MultiAddress::Id(account_id) = address else {
			continue;
		};

		println!("Address: {}, Submitted data: {:?}", account_id, &sd.data[0..3])
	}

	println!("\nEvents");
	let events = client.event_client();
	let block_events = events.block_events(block.header.hash()).await?;
	for raw_event in &block_events {
		let Ok(runtime_event) = RuntimeEvent::try_from(raw_event) else {
			continue;
		};

		println!(
			"Pallet index: {}, Variant index: {}",
			runtime_event.pallet_index(),
			runtime_event.variant_index()
		);

		let RuntimeEvent::DataAvailability(Event::DataSubmitted { who, data_hash }) = &runtime_event else {
			continue;
		};

		println!("Who: {}, Data Hash: {:?}", who, data_hash);
	}

	Ok(())
}
