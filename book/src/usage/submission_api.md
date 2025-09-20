# Transaction Submission API

<!-- langtabs-start -->
```rust
// Establishing a connection
let client = Client::new(TURING_ENDPOINT).await?;

// Defining account that will sign future transaction
let signer = Keypair::from_str("bottom drive obey lake curtain smoke basket hold race lonely fit walk")?;
```
<!-- langtabs-end -->


<!-- langtabs-start -->
```rust
// Transaction Creation
let submittable_tx = client.tx().data_availability().submit_data("My First Data Submission");

// Transaction Submission
let submitted_tx = submittable_tx.sign_and_submit(&signer, Options::new(2)).await?;
println!("Tx Hash: {:?}", submitted_tx.tx_hash);
```

<!-- langtabs-end -->

<!-- langtabs-start -->
```rust
// Transaction Receipt
let receipt = submitted_tx.receipt(false).await?;
let Some(receipt) = receipt else {
    panic!("Oops, looks like our transaction was dropped")
};
println!("Block Hash: {:?}, Block Height: {}", receipt.block_info.hash, receipt.block_info.height);
println!("Tx Hash: {:?}, Tx Index: {}", receipt.tx_info.hash, receipt.tx_info.index);
```

<!-- langtabs-end -->

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