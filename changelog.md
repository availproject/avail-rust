# v0.1.3 Release
- Fixed wasm build

# v0.1.2 Release
- Updated Polkadot SDK version to 12, Avail Core version to 7

# v0.1.1 Release
- Added missing Vector struct to SDK.tx

# v0.1.0 Full Release
Release v0.1.0 (compared to v0.1.0-rc6) introduces numerous breaking changes and fixes. Here is the list:

## Unified Client Instance
Previously, users had to juggle between `OnlineClient`s and `RpcClient`s. Some functions required only one type of client, while others needed both. This caused confusion, unnecessary typing, and hindered optimizations.

Starting from version 0.1.0, this has changed. Now, only a single type of `Client` is used across the entire SDK. The new `Client` structure looks like this:
```rust
#[derive(Debug, Clone)]
pub struct Client {
	pub online_client: AOnlineClient,
	pub rpc_client: RpcClient,
	pub mode: ClientMode,
}
```
It combines `online_client`, `rpc_client`, and a property called `mode`. `ClientMode` determines whether transaction execution and monitoring should be performed using HTTP or WS calls.

This is influenced by the `rpc_client` instance. An HTTP `rpc_client` instance will return an error if the client mode is set to `WS`. Meanwhile, a WS `rpc_client` instance will maintain the connection even if the client mode is set to `HTTP`.

## Unified WS/HTTP Interface
Previously, WS and HTTP traits were used to determine the connection type for transaction execution and monitoring.

Starting from version 0.1.0, all interfaces and functions associated with these interfaces are merged into a single one. The connection type during transaction execution and monitoring is now determined by the value of `Client.mode`.

The default `mode` for `SDK::new_http()` clients is `HTTP`, while the default for `SDK::new()` is `WS`. This can be overridden at any time by changing the value of `Client.mode`.

## Streamlining Transaction Execution
There were four types of transaction execution available to users:
- `execute`
- `execute_and_watch`
- `execute_and_watch_finalization`
- `execute_and_watch_inclusion`

Starting from version 0.1.0, `execute_and_watch` has been removed, and the other `execute_and_*` interfaces, along with `sign_send_and_watch`, have been streamlined as much as possible.

The `sign_send_and_watch` function now uses the new `Logger` instance for logging, reducing code complexity. An improved `Watcher` instance is also introduced, capable of handling both HTTP and WS operations simultaneously.

The parameters for `sign_send_and_watch` have been changed. It no longer accepts `retry_count` or `block_timeout` as inputs. The number of retries is fixed at 2, and transactions are resubmitted after scanning 5 blocks. The watcher only quits when `(best_block_number + 5)` is reached.

In HTTP mode, the client fetches a new header every 3 seconds to check if a new block has been imported by the node. This interval is not adjustable through transaction execution interfaces.

## Streamlining Transaction Watcher
Previously, there was a `watcher` function to check if a transaction hash had been included in the next N blocks. The function was messy and hard to work with.

Starting from version 0.1.0, a dedicated `Watcher` structure exists, allowing for simpler and more fine-tuned search results.

```rust
#[derive(Clone)]
pub struct Watcher {
	client: Client,
	tx_hash: H256,
	wait_for: WaitFor,
	block_count_timeout: Option<u32>,
	block_height_timeout: Option<u32>,
	logger: Arc<Logger>,
	block_fetch_interval: Duration,
	client_mode: ClientMode,
}

impl Watcher {
	pub fn new(client: Client, tx_hash: H256) -> Self {
		Self {
			wait_for: WaitFor::BlockInclusion,
			block_fetch_interval: Duration::from_secs(3),
			client_mode = client.mode,
            ...
		}
	}
```

The default behavior is to wait for block inclusion. The watcher uses the client's current connection mode, with the block fetch interval set to 3 seconds (relevant only for HTTP). The search stops when `best_block_height + 5` is reached. If both block height and block count are set, `block_height_timeout` takes priority.

The logging instance is passive by default. For logging to be enabled, a logger instance with logging enabled must be manually set.

All field members have appropriate setters, making the watcher fully customizable.

## Simplifying Nonce and Mortality
`Nonce` was its own enum type with four variants: `BestBlock`, `FinalizedBlock`, `BestBlockAndTxPool`, and `Custom(u32)`. `Mortality` was a structure containing `period` and `block_hash` fields.

Starting from version 0.1.0, `Nonce` and `Mortality` structures have been removed. Nonce is now defined as `Option<u32>`, defaulting to `BestBlockAndTxPool`. Mortality is defined as `Option<u64>`, defaulting to 32 blocks. Mortality can be set to any value but will be clipped to a power-of-two value. Instead of using best block hash for mortality, it now uses finalized block hash as the former didn't perform as expected.

## New Transaction Payments API
The old transaction payment API used `payment.queryFeeDetails` and `payment.queryInfo` RPCs. Unfortunately, using these required `keyring` and `TransactionOptions`.

Starting from version 0.1.0, all `payment.*` RPCs have been removed in favor of `transaction_payment_api.*` and `transaction_payment_call_api.*` runtime APIs. The former provides the same interface as `payment.*` RPCs, while the latter offers similar calls without requiring keyring and TransactionOptions. This allows users to obtain transaction costs for any constructable payload without additional data.

## Layer of Indirection for Block Transactions and Transaction Events
Previously, block transaction details and transaction events relied on existing methods provided by underlying Subxt structures.

Starting from version 0.1.0, a new layer of abstraction has been introduced for block transactions and transaction events. This simplifies user operations while hiding complexity.

The following abstractions were added:
- `BlockTransactions` - Abstracts away array of `BlockTransaction` 
- `BlockTransaction` - Abstracts away `AExtrinsicDetails`
- `EventRecords` - Abstracts away array of `AEventDetails`

Example: 
```rust
// Printout Block Transactions
for tx in block_transactions.iter() {
    println!(
        "Pallet Name: {:?}, Pallet Index: {}, Call Name: {:?}, Call Index: {:?}, Tx Hash: {:?}, Tx Index: {}",
        tx.pallet_name(),
        tx.pallet_index(),
        tx.call_name(),
        tx.call_index(),
        tx.tx_hash(),
        tx.tx_index()
    );

    println!(
        "Tx Signer: {:?}, App Id: {:?}, Tip: {:?}, Mortality: {:?}, Nonce: {:?}",
        tx.ss58address(),
        tx.app_id(),
        tx.tip(),
        tx.mortality(),
        tx.nonce(),
    );
}

for event in tx_events.iter() {
    let tx_index = match event.phase() {
        subxt::events::Phase::ApplyExtrinsic(x) => Some(x),
        _ => None,
    };

    println!(
        "Pallet Name: {}, Pallet Index: {}, Event Name: {}, Event Index: {}, Event Position: {}, Tx Index: {:?}",
        event.pallet_name(),
        event.pallet_index(),
        event.variant_name(),
        event.variant_index(),
        event.index(),
        tx_index,
    );
}

let event = tx_events.find_first::<NetAccountEvent>();
assert!(event.as_ref().is_some_and(|x| x.is_some()), "NetAccountEvent");
let event = event.unwrap().unwrap();
println!("Account: {}", event.account);
```

There are many more additional quality-of-life methods attached to `BlockTransactions`, `BlockTransaction`, and `EventRecords`.

## Block Transaction and Data Submission Filtering
Previously, block transaction and data submission filtering were handled by dedicated `Block` methods. This was limiting when mixing and matching filters.

Starting from version 0.1.0, there are no dedicated methods for filtering transactions and data submissions. Instead, queries now accept a `Filter` instance, defining which results are discarded.

```rust
#[derive(Debug, Clone, Default)]
pub struct Filter {
	pub app_id: Option<u32>,
	pub tx_hash: Option<H256>,
	pub tx_index: Option<u32>,
	pub tx_signer: Option<AccountId>,
}
```

```rust
let blobs = block.data_submissions(Filter::new().app_id(app_id));
let txs = block.transactions(Filter::new().tx_index(tx_index));
```

## Transaction Execution State
The old transaction execution state interface was confusing as it returned a `Result` inside an `Option`.

Starting from version 0.1.0, the interface has been streamlined to return `Option<bool>`, where `None` indicates that the execution status couldn't be determined. `Some(true)` indicates success, while `Some(false)` indicates failure.
