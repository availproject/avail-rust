# Avail Rust SDK

Rust SDK for interacting with Avail networks.

## Quick Start

`Cargo.toml`:

```toml
[dependencies]
avail-rust-client = { version = "0.5.1", default-features = false, features = ["native", "reqwest"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

For WASM, use `wasm` instead of `native`.

## Example

```rust
use avail_rust_client::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::connect(TURING_ENDPOINT).await?;
    let signer = Keypair::from_str("bottom drive obey lake curtain smoke basket hold race lonely fit walk")?;

    let tx = client
        .tx()
        .data_availability()
        .submit_data(2, "My First Data Submission");

    let submitted = tx.submit_signed(&signer, Options::new()).await?;
    println!(
        "Ext Hash: {:?}, Account: {}, Nonce: {}",
        submitted.ext_hash,
        submitted.account_id,
        submitted.options.nonce
    );

    let receipt = submitted
        .wait_for_receipt(BlockQueryMode::Finalized)
        .await?;

    println!(
        "Block Height: {}, Block Hash: {:?}, Ext Index: {}",
        receipt.block_height,
        receipt.block_hash,
        receipt.ext_index
    );

    Ok(())
}
```

## Features

- `reqwest`: HTTP JSON-RPC transport.
- `tracing`: Structured/plain tracing support.

With `default-features = false`, select either `native` or `wasm`.

## Tracing

Enable the `tracing` feature and initialize once:

```rust
use avail_rust_client::prelude::*;

Client::init_tracing(TracingFormat::Plain)?;
```

Run with logs:

```bash
RUST_LOG=info cargo run
```

## More Examples

See examples in `client/examples`.

## Help

If something is unclear or missing, open a discussion or issue in the Avail repository.

## License

MIT. See `LICENSE`.
