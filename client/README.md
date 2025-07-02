# Avail Rust

**Avail Rust is the Rust library for communicating with Avail networks.**

## In Action

This example uses [Tokio](https://crates.io/crates/tokio) runtime but you can
use any runtime that you like. You `Cargo.toml` file could look like this:

```toml
[dependencies]
avail-rust-client = { version = "0.2.0", default-features = false, features = ["native", "reqwest"] }
tokio = { version = "1.45.0", features = ["rt-multi-thread", "macros"] }
```

> [!NOTE]
> For wasm environment runtime replace "native" with "wasm"

And then the code:
```rust
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	let client = Client::new(TURING_ENDPOINT).await?;

	// Transaction Creation
	let submittable_tx = client.tx().data_availability().submit_data(vec![0, 1, 2, 3, 4, 5]);

	// Transaction Submission
	let submitted_tx = submittable_tx.sign_and_submit(&alice(), Options::new(Some(2))).await?;

	// Fetching Transaction Receipt
	let receipt = submitted_tx.receipt(false).await?;
	let Some(receipt) = receipt else {
		return Err("Transaction got dropped.".into());
	};

	// Fetching Block State
	let block_state = receipt.block_state().await?;
	match block_state {
		BlockState::Included => println!("Block is included but not finalized"),
		BlockState::Finalized => println!("Block is finalized"),
		BlockState::Discarded => println!("Block is discarded"),
		BlockState::DoesNotExist => println!("Block does not exist"),
	}

	// Fetching and displaying Transaction Events
	let event_group = receipt.tx_events().await?;
	for event in event_group.events {
		println!("Pallet Index: {}, Variant index: {}", event.emitted_index.0, event.emitted_index.1);
	}

	Ok(())
}
```
You can find [this](https://github.com/availproject/avail-rust/tree/main/examples/transaction_submission) and as well similar examples in the [example directory](https://github.com/availproject/avail-rust/tree/main/examples).

## Feature Flags
The library is designed to only use the dependencies that it needs so by using `default-features = false, features = []` the library will not compile as the minimum is to choose between the `native` or `wasm` target.

After that you are free to choose one or all of the following feature flags:
- `reqwest`: Sets up a basic rpc client that can be used to send and receive network data. If you are unsure what that means then it's best to add this feature flag to your own list.
- `tracing`: Enables logging/tracing. Useful when dealing with nonce and other transaction related issues. The output of the logging library can be set to json format if necessary.
- `subxt`: Gives access to the whole external subxt library. Subxt can be useful when you need to fetch and deal with storage and constants related data.
- `generated_metadata`: Gives access to all possible extrinsics, events and other chain related metadata types. By default, a subset of metadata types is already available to be used, but if really necessary this feature flag will provide you with everything. Use this feature with caution as it will increase the compilation time dramatically (by over 10s) and cause the rust-analyzer to sometimes give up on analyzing your code. If a metadata type is not available, it's best to manually define as shown in [custom transaction](https://github.com/availproject/avail-rust/tree/main/examples/custom_transaction) and [custom event](https://github.com/availproject/avail-rust/tree/main/examples/custom_event) examples.

## Examples
All existing and new examples can be found [here](https://github.com/availproject/avail-rust/tree/main/examples). They cover most of the basic needs and interactions with the chain, but if there is something that you need and it's not covered by the example let us know and we might add it :)

Here is the incomplete list of the examples that we currently have:
- [Batching transactions](https://github.com/availproject/avail-rust/tree/main/examples/batch)
- [Executing transactions in parallel](https://github.com/availproject/avail-rust/tree/main/examples/parallel_transaction_submission)
- Writing your own custom [transaction](https://github.com/availproject/avail-rust/tree/main/examples/custom_transaction) or [event](https://github.com/availproject/avail-rust/tree/main/examples/custom_event)
- Dealing with [blocks](https://github.com/availproject/avail-rust/tree/main/examples/block_client) and [events](https://github.com/availproject/avail-rust/tree/main/examples/custom_event)
- [A full example on how to submit a transaction from beginning till the end](https://github.com/availproject/avail-rust/tree/main/examples/transaction_submission)
- [Subscribing to block headers, blocks and justifications](https://github.com/availproject/avail-rust/tree/main/examples/transaction_submission)

## Logging/Tracing
In order to enable it you need to use the `tracing` feature flag and in the code call `Client::enable_tracing(boolean);` where boolean is either true or false depending if you want to enable or not json format structured logging.

Example:
```toml
[dependencies]
avail-rust-client = { version = "0.2.0", default-features = false, features = ["native", "reqwest", "tracing"] }
tokio = { version = "1.45.0", features = ["rt-multi-thread", "macros"] }
```

```rust
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	Client::enable_tracing(false);
	let client = Client::new(LOCAL_ENDPOINT).await?;
    ...
}
```

## Custom Transaction and Custom Events
Sometimes you need a specific transaction or event type that is not available in the default metadata but at the same time you don't want to enable the  `generated_metadata` feature flag as it will your kill compile time. In case like that what you can do is define a custom transaction or event and use it normally as you would like any other predefined type. Both custom transaction and custom events are super easy to implement and use.

Create custom transaction:
```rust
use avail_rust_client::{
	avail::{TransactionCallLike, TxDispatchIndex},
	prelude::*,
};

#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomTransaction {
	pub data: Vec<u8>,
}
impl TxDispatchIndex for CustomTransaction {
	const DISPATCH_INDEX: (u8, u8) = (29u8, 1u8);
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
	let client = Client::new(LOCAL_ENDPOINT).await?;

	let custom_tx = CustomTransaction { data: vec![0, 1, 2, 3] };
	let submittable = custom_tx.to_submittable(client.clone());
	let submitted = submittable.sign_and_submit(&alice(), Options::new(Some(2))).await?;
	let receipt = submitted.receipt(true).await?.expect("Must be there");
}
```

Create custom event:
```rust
use avail_rust_client::{
	avail::{TransactionEventLike, TxEventEmittedIndex},
	prelude::*,
};

#[derive(codec::Decode, codec::Encode, PartialEq, Eq)]
pub struct CustomEvent {
	pub who: AccountId,
	pub data_hash: H256,
}
impl TxEventEmittedIndex for CustomEvent {
	const EMITTED_INDEX: (u8, u8) = (29, 1);
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // For brevity reasons the way of getting the encoded event has been skipped but
    // in a nutshell you can get from the event client or from calling receipt.tx_events()
	let encoded_event = vec![0, 1, 2, 3];
	let event = CustomEvent::from_raw(&encoded_event).expect("Must be Ok");
	println!("Account: {}, Hash: {}", event.who, event.data_hash);

	Ok(())
}
```

## Getting help
In the `avail-rust`s repo we have a number of examples showing how to utilize this library. If something is missing or you have a question don't hesitate to open a [discussion](https://github.com/availproject/avail-rust/discussions) with your question or reach us on [discord](https://www.availproject.org/developer) (the link to our discord is at the bottom of the page).


## Contribution
Thank you for being interested in improving this project! Right now as we are still add new features and trying to finalize existing ones things or would be beneficial if you first post your idea in the [discussions](https://github.com/availproject/avail-rust/discussions) or in the [issues](https://github.com/availproject/avail-rust/issues).

Pull requests which fix grammatical mistakes, cargo clippy related warnings or in any way don't provide anything useful will be closed immediately without any feedback.