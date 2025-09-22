# Chain API

<!-- langtabs-start -->
```rust
// Block Hash
let best = client.best().block_hash().await?;
let finalized = client.finalized().block_hash().await?;
let block_hash = client.rpc().block_hash(Some(1900000)).await?.expect("Should be there");

// Block Height
let best = client.best().block_height().await?;
let finalized = client.finalized().block_height().await?;
let block_height = client.rpc().block_height(block_hash).await?.expect("Should be there");
println!("Best: {}, Finalized: {}, Specific: {}", best, finalized, block_height);
```
<!-- langtabs-end -->

<!-- langtabs-start -->
```rust
// Block Info
let best = client.best().block_info().await?;
let finalized = client.finalized().block_info().await?;
println!("Best Hash: {:?}, Height: {}", best.hash, best.height);
println!("Finalized Hash: {:?}, Height: {}", finalized.hash, finalized.height);

// Chain Info
let chain_info = client.rpc().chain_info().await?;
println!("Best Hash: {:?}, Height: {}", chain_info.best_hash, chain_info.best_height);
println!("Finalized Hash: {:?}, Height: {}", chain_info.finalized_hash, chain_info.finalized_height);
println!("Genesis Hash: {:?}", chain_info.genesis_hash);
```
<!-- langtabs-end -->

<!-- langtabs-start -->
```rust
// Block State
let block_state = client.rpc().block_state(1900000).await?;
match block_state {
    BlockState::Included => println!("Block Not Yet Finalized"),
    BlockState::Finalized => println!("Block Finalized"),
    BlockState::Discarded => println!("Block Discarded"),
    BlockState::DoesNotExist => println!("Block Does not Exist"),
};

// Block Header
let at = Some(1900000);
let best = client.best().block_header().await?;
let finalized = client.finalized().block_header().await?;
let specific = client.rpc().block_header(at).await?.expect("Should be there");
println!("Best Header: Hash: {:?}, Height: {}", best.hash(), best.number);
println!("Finalized Header: Hash: {:?}, Height: {}", finalized.hash(), finalized.number);
println!("Specific Header: Hash: {:?}, Height: {}", specific.hash(), specific.number);
```
<!-- langtabs-end -->

<!-- langtabs-start -->
```rust
// Account Nonces
let address = "5Ev16A8iWsEBFgtAxcyS8T5nDx8rZxWkg2ZywPgjup3ACSUZ";
let best = client.best().account_nonce(address).await?;
let finalized = client.finalized().account_nonce(address).await?;
let specific = client.rpc().block_nonce(address, 1000000).await?;
// RPC nonce is the one that you want 99.99% of time
let rpc = client.rpc().account_nonce(address).await?;
println!("Best Nonce: {}, Finalized Nonce: {}, Specific Nonce: {},", best, finalized, specific);
println!("RPC Nonce: {}", rpc);

// Account Balances
let address = "5FjdibsxmNFas5HWcT2i1AXbpfgiNfWqezzo88H2tskxWdt2";
let best = client.best().account_balance(address).await?;
let finalized = client.finalized().account_balance(address).await?;
let specific = client.rpc().account_balance(address, 1000000).await?;
println!(
    "Best Free Balance: {}, Finalized Free Balance: {}, Specific Free Balance: {}",
    best.free, finalized.free, specific.free
);
```
<!-- langtabs-end -->

<!-- langtabs-start -->
```rust
// Account Info
let address = "5GReLENC89bZfEQdytoMDY2krPnX1YC3qe14Gj3zFbjov4hX";
let best = client.best().account_info(address).await?;
let finalized = client.finalized().account_info(address).await?;
let specific = client.rpc().account_info(address, 1000000).await?;
println!("Best: Nonce: {},  Free Balance: {}", best.nonce, best.data.free);
println!("Finalized: Nonce: {},  Free Balance: {}", finalized.nonce, finalized.data.free);
println!("Specific: Nonce: {},  Free Balance: {}", specific.nonce, specific.data.free);
```
<!-- langtabs-end -->

## Source Code

<!-- langtabs-start -->
```rust
{{#include ../../../examples/chain_api/src/main.rs}}
```
<!-- langtabs-end -->
