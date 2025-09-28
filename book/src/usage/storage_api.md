# Storage API

The Storage API lets you query on-chain storage entries directly, whether
they’re simple values, maps, double maps or iterable key-value pairs.

> 📝 Note: Not all pallet storages are defined in the SDK. Let us know
> if the one you need is missing and we will add it in no time.

#### Connect to the Chain

<!-- langtabs-start -->

```rust
// Establishing a connection
let client = Client::new(TURING_ENDPOINT).await?;
let rpc_client = &client.rpc_client;
```

<!-- langtabs-end -->

#### Fetch Storage Value (example: DataAvailability::NextAppId)

<!-- langtabs-start -->

```rust
// Fetching DataAvailability::NextAppId - Storage Value
let next_app_id = DAStorage::NextAppId::fetch(rpc_client, None)
    .await?
    .expect("Should be there");
println!("Next App Id: {}", next_app_id.0);
```

<!-- langtabs-end -->

#### Fetch Storage Map (example: DataAvailability::AppKeys)

<!-- langtabs-start -->

```rust
// Fetching DataAvailability::AppKeys - Storage Map
let key = "Hello World".as_bytes().to_vec();
let app_key = DAStorage::AppKeys::fetch(rpc_client, &key, None)
    .await?
    .expect("Should be there");
println!("AppKey Owner: {}, AppKey Id: {}", app_key.owner, app_key.id);
```

<!-- langtabs-end -->

#### Iterate Over Storage Map (example: DataAvailability::AppKeys)

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

#### Fetch Storage Map (example: System::Account)

<!-- langtabs-start -->

```rust
// Fetching System::Account - Storage Map
let account_id = AccountId::from_str("5Ev2jfLbYH6ENZ8ThTmqBX58zoinvHyqvRMvtoiUnLLcv1NJ").expect("Should work");
let account_info = SystemStorage::Account::fetch(&client.rpc_client, &account_id, None)
    .await?
    .expect("Should be there");
println!("Account Nonce: {}, Account Free Balance: {}", account_info.nonce, account_info.data.free);
```

<!-- langtabs-end -->

#### Iterate Over Storage Map (example: System::Account)

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
