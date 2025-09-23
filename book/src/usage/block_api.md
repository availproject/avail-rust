# Block API

The Block API lets you query blocks, inspect transactions (extrinsics), decode
events, and check finality justifications.

> 📝 Note: A block_id can be provided as a block number, hash, or string.

#### Connect to a block

<!-- langtabs-start -->

```rust
// Establishing a connection
let client = Client::new(MAINNET_ENDPOINT).await?;

// block_id can be a number, string or hash
let block = client.block(1913231);
// or -> let block = Block::new(client.clone(), 1913231);
```

<!-- langtabs-end -->

#### Query Specific Transactions (Signed Extrinsics)

<!-- langtabs-start -->

```rust
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
```

<!-- langtabs-end -->

#### Query Specific Signed or Unsigned Extrinsics

<!-- langtabs-start -->

```rust
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
```

<!-- langtabs-end -->

#### Work With Raw Extrinsics

<!-- langtabs-start -->

```rust
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
```

<!-- langtabs-end -->

#### Fetch Extrinsic-related Events

<!-- langtabs-start -->

```rust
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
```

<!-- langtabs-end -->

#### Fetch Block Events

<!-- langtabs-start -->

```rust
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
```

<!-- langtabs-end -->

#### Fetch Grandpa Justifications

<!-- langtabs-start -->

```rust
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
```

<!-- langtabs-end -->

## Full Example

<!-- langtabs-start -->

```rust
{{#include ../../../examples/block_api/src/main.rs}}
```

<!-- langtabs-end -->
