# Avail Rust

**Avail Rust is a Rust library for communicating with Avail networks.**

## In Action

This example uses the [Tokio](https://crates.io/crates/tokio) runtime, but you
can use any runtime you like. Your `Cargo.toml` file could look like this:

```toml
[dependencies]
avail-rust-client = { version = "0.4.0-rc.3", default-features = false, features = ["native", "reqwest"] }
tokio = { version = "1.45.0", features = ["rt-multi-thread", "macros"] }
```

> [!NOTE]
> For the WASM environment, replace "native" with "wasm".

And then the code:

```rust
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
	// Establishing a connection
	let client = Client::new(TURING_ENDPOINT).await?;

	// Defining account that will sign future transaction
	let signer = Keypair::from_str("bottom drive obey lake curtain smoke basket hold race lonely fit walk")?;
	// Or use one of dev accounts -> let signer = alice();

	// Transaction Creation
	let submittable = client.tx().data_availability().submit_data("My First Data Submission");

	// Transaction Submission
	let submitted = submittable.sign_and_submit(&signer, Options::new(2)).await?;
	println!(
		"Tx Hash: {:?}, Account Id: {}, Nonce: {}, App Id: {}",
		submitted.tx_hash, submitted.account_id, submitted.options.nonce, submitted.options.app_id
	);

	// Transaction Receipt
	let receipt = submitted.receipt(false).await?;
	let Some(receipt) = receipt else {
		panic!("Oops, looks like our transaction was dropped")
	};
	println!(
		"Block Height: {}, Block Hash: {:?}, Tx Hash: {:?}, Tx Index: {}",
		receipt.block_ref.height, receipt.block_ref.hash, receipt.tx_ref.hash, receipt.tx_ref.index
	);

	let block_state = receipt.block_state().await?;
	match block_state {
		BlockState::Included => println!("Block Not Yet Finalized"),
		BlockState::Finalized => println!("Block Finalized"),
		BlockState::Discarded => println!("Block Discarded"),
		BlockState::DoesNotExist => println!("Block Does not Exist"),
	};

	Ok(())
}
```

You can find
[this](https://github.com/availproject/avail-rust/tree/main/client/examples/submission.rs)
example and similar ones in the
[example directory](https://github.com/availproject/avail-rust/tree/main/client/examples).

## Feature Flags

The library is designed to use only the necessary dependencies, so by using
`default-features = false, features = []`, the library will not compile unless
you choose either the `native` or `wasm` target.

After that, you are free to choose one or all of the following feature flags:

- `reqwest`: Sets up a basic RPC client for sending and receiving network data.
  If you’re unsure what this means, it’s best to add this feature flag to your
  list.
- `tracing`: Enables logging/tracing, which is useful when dealing with nonce
  and other transaction-related issues. The logging output can be set to JSON
  format if needed.

## Examples

All existing and new examples can be found
[here](https://github.com/availproject/avail-rust/tree/main/client/examples). They
cover most basic needs and interactions with the chain. If there is something
you need that isn’t covered, let us know—we might add it. :)

## Logging/Tracing

To enable tracing, use the `tracing` feature flag and call
`Client::init_tracing(boolean)` in your code, where `boolean` is `true` or
`false` depending on whether you want to enable JSON-format structured logging.

Example:

```toml
[dependencies]
avail-rust-client = { version = "0.4.0-rc.3", default-features = false, features = ["native", "reqwest", "tracing"] }
tokio = { version = "1.45.0", features = ["rt-multi-thread", "macros"] }
```

```rust
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Client::init_tracing(false)?;
    let client = Client::new(LOCAL_ENDPOINT).await?;
    ...
}
```

After everything is set up run the following command:

```bash
RUST_LOG=info cargo run
```

## Getting Help

In the `avail-rust` repository, we have many examples showing how to use this
library. If something is missing or unclear, don't hesitate to open a
[discussion](https://github.com/availproject/avail-rust/discussions) or reach
out to us on [Discord](https://www.availproject.org/developer) (the link is at
the bottom of the page).

## Contribution

Thank you for your interest in improving this project! As we are still adding
new features and finalizing existing ones, it would be helpful to first post
your idea in the
[discussions](https://github.com/availproject/avail-rust/discussions) or
[issues](https://github.com/availproject/avail-rust/issues).

# License

This project is primarily distributed under the terms of MIT license.
See [LICENSE](https://github.com/availproject/avail-rust/blob/main/LICENSE)
