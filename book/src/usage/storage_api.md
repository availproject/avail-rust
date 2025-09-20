# Storage API

<!-- langtabs-start -->
```rust
// Establishing a connection
let client = Client::new(TURING_ENDPOINT).await?;
let rpc_client = &client.rpc_client;
```
<!-- langtabs-end -->

<!-- langtabs-start -->
```rust
// Fetching DataAvailability::AppKeys - Simple storage
let key = "Hello World".as_bytes().to_vec();
let app_key = DAStorage::AppKeys::fetch(rpc_client, &key, None)
    .await?
    .expect("Should be there");
println!("AppKey Owner: {}, AppKey Id: {}", app_key.owner, app_key.id);
```
<!-- langtabs-end -->

<!-- langtabs-start -->
```rust
// Iterating over AppKeys
let block_hash = client.finalized().block_hash().await?;
let mut iter = DAStorage::AppKeys::iter(rpc_client.clone(), block_hash);
for _ in 0..3 {
    // You can fetch just the value...
    let app_key = iter.next().await?.expect("Must be there");
    println!("AppKey Owner: {}, AppKey Id: {}", app_key.owner, app_key.id);

    // ...or both the value and the key
    let (key, app_key) = iter.next_key_value().await?.expect("Must be there");
    println!(
        "AppKey Key: {}, AppKey Owner: {}, AppKey Id: {}",
        String::from_utf8(key).expect("Should work"),
        app_key.owner,
        app_key.id
    );
}
```
<!-- langtabs-end -->

<!-- langtabs-start -->
```rust
// Fetching System::Account - Map storage
let account_id = AccountId::from_str("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ").expect("Should work");
let account_info = SystemStorage::Account::fetch(&client.rpc_client, &account_id, None)
    .await?
    .expect("Should be there");
println!("Account Nonce: {}, Account Free Balance: {}", account_info.nonce, account_info.data.free);
```
<!-- langtabs-end -->

<!-- langtabs-start -->
```rust
// Iterating over Accounts
let mut iter = SystemStorage::Account::iter(client.rpc_client.clone(), client.finalized().block_hash().await?);
for _ in 0..3 {
    // You can fetch just the value...
    let account_info = iter.next().await?.expect("Must be there");
    println!("Account Nonce: {}, Account Free Balance: {}", account_info.nonce, account_info.data.free);

    // ...or both the value and the key
    let (account_id, account_info) = iter.next_key_value().await?.expect("Must be there");
    println!(
        "Account Id: {}, Account Nonce: {}, Account Free Balance: {}",
        account_id, account_info.nonce, account_info.data.free
    );
}
```
<!-- langtabs-end -->



## Full Example
<!-- langtabs-start -->
```rust
{{#include ../../../examples/storage_api/src/main.rs}}
```
<!-- langtabs-end -->
