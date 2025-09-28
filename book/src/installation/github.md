# Github

You can install the SDK directly from Github. This example shows how to add it
to your project and connect to the Avail mainnet.

### 1. Create a new Rust project

```bash
cargo new crates-example
cd crates-example
```

### 2. Add dependencies

Update your Cargo.toml to include the SDK and runtime dependencies:

<!-- langtabs-start -->

```rust
{{#include ../../../examples/github/Cargo.toml}}
```

<!-- langtabs-end -->

### 3. Connect to the chain

Replace the contents of src/main.rs with the following:

<!-- langtabs-start -->

```rust
{{#include ../../../examples/github/src/main.rs}}
```

<!-- langtabs-end -->

### 4. Run the example

```bash
cargo run
```

if everything works, you should see output similar to:

```txt
You are connected to https://mainnet-rpc.avail.so/rpc. Genesis Hash: 0xb91746b45e0346cc2f815a520b9c6cb4d5c0902af848db0a80f85932d2e8276a
```
