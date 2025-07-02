# Avail Rust

**Avail Rust is a Rust library for communicating with Avail networks.**

## In Action

This example uses the [Tokio](https://crates.io/crates/tokio) runtime, but you can
use any runtime you like. Your `Cargo.toml` file could look like this:

```toml
[dependencies]
avail-rust-client = { version = "0.2.0", default-features = false, features = ["native", "reqwest"] }
tokio = { version = "1.45.0", features = ["rt-multi-thread", "macros"] }
```

> [!NOTE]
> For the WASM environment, replace "native" with "wasm".

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

You can find [this](https://github.com/availproject/avail-rust/tree/main/examples/transaction_submission) example and similar ones in the [example directory](https://github.com/availproject/avail-rust/tree/main/examples).

## Feature Flags
The library is designed to use only the necessary dependencies, so by using `default-features = false, features = []`, the library will not compile unless you choose either the `native` or `wasm` target.

After that, you are free to choose one or all of the following feature flags:
- `reqwest`: Sets up a basic RPC client for sending and receiving network data. If you’re unsure what this means, it’s best to add this feature flag to your list.
- `tracing`: Enables logging/tracing, which is useful when dealing with nonce and other transaction-related issues. The logging output can be set to JSON format if needed.
- `subxt`: Provides access to the entire external Subxt library. This can be useful when you need to fetch and manage storage and constants-related data.
- `generated_metadata`: Provides access to all possible extrinsics, events, and other chain-related metadata types. By default, a subset of metadata types is already available, but if necessary, this feature flag gives access to everything. Use this feature with caution—it significantly increases compilation time (by over 10 seconds) and may cause rust-analyzer to stop analyzing your code. If a metadata type isn’t available, it’s best to define it manually, as shown in the [custom transaction](https://github.com/availproject/avail-rust/tree/main/examples/custom_transaction) and [custom event](https://github.com/availproject/avail-rust/tree/main/examples/custom_event) examples.

## Examples
All existing and new examples can be found [here](https://github.com/availproject/avail-rust/tree/main/examples). They cover most basic needs and interactions with the chain. If there is something you need that isn’t covered, let us know—we might add it. :)

Here is an incomplete list of current examples:
- [Batching transactions](https://github.com/availproject/avail-rust/tree/main/examples/batch)
- [Executing transactions in parallel](https://github.com/availproject/avail-rust/tree/main/examples/parallel_transaction_submission)
- Writing your own custom [transaction](https://github.com/availproject/avail-rust/tree/main/examples/custom_transaction) or [event](https://github.com/availproject/avail-rust/tree/main/examples/custom_event)
- Dealing with [blocks](https://github.com/availproject/avail-rust/tree/main/examples/block_client) and [events](https://github.com/availproject/avail-rust/tree/main/examples/custom_event)
- [A full example on how to submit a transaction from start to finish](https://github.com/availproject/avail-rust/tree/main/examples/transaction_submission)
- [Subscribing to block headers, blocks, and justifications](https://github.com/availproject/avail-rust/tree/main/examples/transaction_submission)

## Logging/Tracing
To enable tracing, use the `tracing` feature flag and call `Client::enable_tracing(boolean)` in your code, where `boolean` is `true` or `false` depending on whether you want to enable JSON-format structured logging.

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

## Custom Transactions and Events
Sometimes you need a specific transaction or event type not included in the default metadata. However, enabling the `generated_metadata` feature flag may greatly increase compile time. In such cases, you can define a custom transaction or event and use it just like any predefined type. Both are simple to implement and use.

Create a custom transaction:
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

Create a custom event:
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
    // For brevity, the method of obtaining the encoded event is omitted.
    // In short, you can get it from the event client or from receipt.tx_events().
    let encoded_event = vec![0, 1, 2, 3];
    let event = CustomEvent::from_raw(&encoded_event).expect("Must be Ok");
    println!("Account: {}, Hash: {}", event.who, event.data_hash);

    Ok(())
}
```

## Getting Help
In the `avail-rust` repository, we have many examples showing how to use this library. If something is missing or unclear, don't hesitate to open a [discussion](https://github.com/availproject/avail-rust/discussions) or reach out to us on [Discord](https://www.availproject.org/developer) (the link is at the bottom of the page).

## Contribution
Thank you for your interest in improving this project! As we are still adding new features and finalizing existing ones, it would be helpful to first post your idea in the [discussions](https://github.com/availproject/avail-rust/discussions) or [issues](https://github.com/availproject/avail-rust/issues).

Pull requests that only fix grammatical mistakes, resolve cargo clippy warnings, or do not add any substantial value will be closed immediately without feedback.
