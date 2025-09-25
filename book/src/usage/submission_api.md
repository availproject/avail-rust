# Transaction Submission API

Use the Submission API to connect to a node, create a transaction, sign it,
submit it to the network, and then track its inclusion/finality.

<div class="warning">
Security note

The seed phrase below is a well-known test mnemonic. Never use it (or any
mnemonic) from docs in production.

</div>

#### Connect & choose a signer

<!-- langtabs-start -->

```rust
// Establishing a connection
let client = Client::new(TURING_ENDPOINT).await?;

// Defining account that will sign future transaction
let signer = Keypair::from_str("bottom drive obey lake curtain smoke basket hold race lonely fit walk")?;
```

<!-- langtabs-end -->

- `Client::new(...)` initializes a connection to your chosen RPC endpoint.
  TURING_ENDPOINT will connect you to Avail Turing chain.
- `Keypair::from_str(...)` loads your signing key. In real apps, load keys
  securely (env vars, keystores, etc).

#### Build & submit a transaction

<!-- langtabs-start -->

```rust
// Transaction Creation
let submittable_tx = client.tx().data_availability().submit_data("My First Data Submission");

// Transaction Submission
let submitted_tx = submittable_tx.sign_and_submit(&signer, Options::new(2)).await?;
println!("Tx Hash: {:?}", submitted_tx.tx_hash);
```

<!-- langtabs-end -->

- `submit_data(...)` constructs the payload.
- `sign_and_submit(...)` signs with your signer and broadcasts to the network.
- `Options::new(2)` configures submission options (e.g. `2` means that the App
  ID will be set to 2).

#### Get a receipt

<!-- langtabs-start -->

```rust
// Transaction Receipt
let receipt = submitted_tx.receipt(false).await?;
let Some(receipt) = receipt else {
    panic!("Oops, looks like our transaction was dropped")
};
println!("Block Hash: {:?}, Block Height: {}", receipt.block_ref.hash, receipt.block_ref.height);
println!("Tx Hash: {:?}, Tx Index: {}", receipt.tx_ref.hash, receipt.tx_ref.index);
```

<!-- langtabs-end -->

- `receipt(false)` retrieves inclusion info without waiting for finality
- If `None`, the node didn’t observe inclusion (probably dropped). Consider
  retry

#### Track block state (inclusion vs. finality)

<!-- langtabs-start -->

```rust
// Transaction Block State
let block_state = receipt.block_state().await?;
match block_state {
    BlockState::Included => println!("Block Not Yet Finalized"),
    BlockState::Finalized => println!("Block Finalized"),
    BlockState::Discarded => println!("Block Discarded"),
    BlockState::DoesNotExist => println!("Block Does not Exist"),
};
```

<!-- langtabs-end -->

Typical meanings:

- **Included** – your tx made it into a block but isn’t finalized yet.
- **Finalized** – the block (and your tx) is finalized.
- **Discarded** – the block was orphaned/reorged out.
- **DoesNotExist** – the referenced block can’t be found

## Full Example

<!-- langtabs-start -->

```rust
{{#include ../../../examples/submission_api/src/main.rs}}
```

<!-- langtabs-end -->
