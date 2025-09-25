use avail_rust::{
	avail::{
		data_availability::{events::DataSubmitted, tx::SubmitData},
		timestamp::tx::Set,
	},
	prelude::*,
};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
	// Establishing a connection
	let client = Client::new(MAINNET_ENDPOINT).await?;

	// block_id can be a number, string or hash
	let block = client.block(1913231);
	// or -> let block = Block::new(client.clone(), 1913231);

	// Fetching all transactions (signed extrinsics) of type DataAvailability::SubmitData
	let all_submit_data = block.tx().all::<SubmitData>(Default::default()).await?;
	for tx in all_submit_data {
		// Displaying transaction general and specific information
		let (app_id, address, data_len) = (tx.app_id(), tx.ss58_address(), tx.call.data.len());
		println!("App ID: {}, SS58 Address: {:?}, Data Length: {} bytes", app_id, address, data_len);

		// Fetching transaction events
		let events = tx.events(client.clone()).await?;
		let event = events.first::<DataSubmitted>().expect("Should be present");
		println!("Who: {}, Data Hash: {:?}", event.who, event.data_hash)
	}
	/*
	   App ID: 25, SS58 Address: Some("5Ck9e4wm38Kdmap82aYrSeJ7MKfxXgagqfFdUjCpHXbFMjst"), Data Length: 262194 bytes
	   Who: 5Ck9e4wm38Kdmap82aYrSeJ7MKfxXgagqfFdUjCpHXbFMjst, Data Hash: 0x50dd8f9b2b95a2809ae375a60599feaa90f3be8461a510363f5c0e03b724a41e
	   App ID: 25, SS58 Address: Some("5EHoP9SFQ9N1RrWAeWuLQG2ngHwChgSCNgtUm31p5XpjhXDq"), Data Length: 262194 bytes
	   Who: 5EHoP9SFQ9N1RrWAeWuLQG2ngHwChgSCNgtUm31p5XpjhXDq, Data Hash: 0xe069e11254ed2e6958acdc8f9f79287e5046c1c4eb650eeec409bbd79a0f45d1
	*/

	// Fetching extrinsic (signed or unsigned) of type Timestamp::Set at index 0
	let first_extrinsic = block.ext().get::<Set>(0).await?;
	if let Some(ext) = first_extrinsic {
		// Displaying extrinsic general and specific information
		let (app_id, address, timestamp) = (ext.app_id(), ext.ss58_address(), ext.call.now);
		println!("App ID: {:?}, SS58 Address: {:?}, Timestamp: {}", app_id, address, timestamp);

		// Fetching extrinsic events
		let events = ext.events(client.clone()).await?;
		assert!(events.is_extrinsic_success_present());
	}
	/*
	   App ID: None, SS58 Address: None, Timestamp: 1758361080000
	*/

	// Fetching all extrinsics (singed or unsigned) in raw format.
	// Raw format means that they are not decoded and we need to do it manually.
	let all_extrinsics = block.raw_ext().all(Default::default()).await?;
	for raw_ext in all_extrinsics {
		let id = (raw_ext.metadata.pallet_id, raw_ext.metadata.variant_id);
		if id == SubmitData::HEADER_INDEX {
			println!("Found submit data transaction");

			// We can convert Block Raw Extrinsic directly to Block Transaction...

			let tx = BlockTransaction::<SubmitData>::try_from(raw_ext.clone()).expect("Should be decodable");
			let (app_id, address, data_len) = (tx.app_id(), tx.ss58_address(), tx.call.data.len());
			println!("App ID: {}, SS58 Address: {:?}, Data Length: {} bytes", app_id, address, data_len);

			//... or directly to the correct call.
			let call =
				SubmitData::from_ext(raw_ext.data.as_ref().expect("Should be there")).expect("Should be decodable");
			println!("Data Length: {} bytes", call.data.len())
		}

		if id == Set::HEADER_INDEX {
			println!("Found timestamp set extrinsic");
		}
	}
	/*
		Found timestamp set extrinsic
		Found submit data transaction
		App ID: 25, SS58 Address: Some("5Ck9e4wm38Kdmap82aYrSeJ7MKfxXgagqfFdUjCpHXbFMjst"), Data Length: 262194 bytes
		Data Length: 262194 bytes
		Found submit data transaction
		App ID: 25, SS58 Address: Some("5EHoP9SFQ9N1RrWAeWuLQG2ngHwChgSCNgtUm31p5XpjhXDq"), Data Length: 262194 bytes
		Data Length: 262194 bytes
	*/

	// Fetching extrinsic related events
	let ext_events = block.events().ext(1).await?.expect("Should be there");
	let event = ext_events.first::<DataSubmitted>().expect("Should be present");
	println!("Who: {}, Data Hash: {:?}", event.who, event.data_hash);

	for event in &ext_events.events {
		// Displaying event general information
		let (pallet_id, variant_id, index, data_len) =
			(event.pallet_id, event.variant_id, event.index, event.data.len());
		println!(
			"Event Pallet Id: {}, Variant Id: {}, Index: {}, Data len: {} bytes",
			pallet_id, variant_id, index, data_len
		);

		// Converting generic data to a specific event type
		if (pallet_id, variant_id) == DataSubmitted::HEADER_INDEX {
			println!("Found Data Submitted event");
			let data_submitted = DataSubmitted::from_event(&event.data).expect("Should be decodable");
			println!("Who: {}, Data Hash: {:?}", data_submitted.who, data_submitted.data_hash)
		}
	}
	/*
		Who: 5Ck9e4wm38Kdmap82aYrSeJ7MKfxXgagqfFdUjCpHXbFMjst, Data Hash: 0x50dd8f9b2b95a2809ae375a60599feaa90f3be8461a510363f5c0e03b724a41e
		Event Pallet Id: 6, Variant Id: 7, Index: 3, Data len: 100 bytes
		Event Pallet Id: 6, Variant Id: 7, Index: 4, Data len: 100 bytes
		Event Pallet Id: 6, Variant Id: 7, Index: 5, Data len: 100 bytes
		Event Pallet Id: 7, Variant Id: 0, Index: 6, Data len: 132 bytes
		Event Pallet Id: 0, Variant Id: 0, Index: 7, Data len: 36 bytes
	*/

	// Fetching all events from a block.
	// The events are in raw format which means that they are not decoded.
	let block_events = block.events().block(Default::default()).await?;
	for phase_event in &block_events {
		// Displaying phase
		println!("Phase: {:?}", phase_event.phase);

		for event in &phase_event.events {
			// Displaying event general information
			let (pallet_id, variant_id, index) = (event.pallet_id, event.variant_id, event.index);
			println!("Event Pallet Id: {}, Variant Id: {}, Index: {}", pallet_id, variant_id, index);

			if let Some(_data) = &event.encoded_data {
				// Do something with event data.
			} else {
				// No event data was requested so none was send in response.
			}
		}
	}
	/*
		Phase: ApplyExtrinsic(0)
		Event Pallet Id: 0, Variant Id: 0, Index: 0
		Phase: ApplyExtrinsic(1)
		Event Pallet Id: 6, Variant Id: 8, Index: 1
		Event Pallet Id: 29, Variant Id: 1, Index: 2
		Event Pallet Id: 6, Variant Id: 7, Index: 3
		Event Pallet Id: 6, Variant Id: 7, Index: 4
		Event Pallet Id: 6, Variant Id: 7, Index: 5
		Event Pallet Id: 7, Variant Id: 0, Index: 6
		Event Pallet Id: 0, Variant Id: 0, Index: 7
		Phase: ApplyExtrinsic(2)
		Event Pallet Id: 6, Variant Id: 8, Index: 8
		Event Pallet Id: 29, Variant Id: 1, Index: 9
		Event Pallet Id: 6, Variant Id: 7, Index: 10
		Event Pallet Id: 6, Variant Id: 7, Index: 11
		Event Pallet Id: 6, Variant Id: 7, Index: 12
		Event Pallet Id: 7, Variant Id: 0, Index: 13
		Event Pallet Id: 0, Variant Id: 0, Index: 14
		Phase: ApplyExtrinsic(3)
		Event Pallet Id: 0, Variant Id: 0, Index: 15
	*/

	// Fetching grandpa justification
	let justification = block.justification().await?;
	if justification.is_some() {
		println!("Justification was found at block: {}", 1913231)
	} else {
		println!("No justification was found at block: {}", 1913231)
	}

	let block = client.block(1913216);
	let justification = block.justification().await?;
	if justification.is_some() {
		println!("Justification was found at block: {}", 1913216)
	} else {
		println!("No justification was found at block: {}", 1913216)
	}
	/*
		No justification was found at block: 1913231
		Justification was found at block: 1913216
	*/

	Ok(())
}
