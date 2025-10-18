# Working with Subscriptions

The SDK exposes a set of subscription helpers that let you iterate over blocks,
extrinsics, transactions, events, and GRANDPA justifications without managing
polling loops manually. This guide covers the main entry points and best
practices.

## Core Concepts

- `Sub` – lazy subscription factory that tracks either the finalized or best
  chain. It controls the polling interval, starting height, and retry policy.
- `BlockSub`, `LegacyBlockSub` – stream block handles or legacy blocks.
- `BlockEventsSub`, `TransactionSub`, `ExtrinsicSub`, `EncodedExtrinsicSub` – stream
  filtered data per block.
- `GrandpaJustification{,Json}Sub` – stream GRANDPA justifications.

Each helper reuses the `Sub` cursor internally and therefore inherits its
configuration.

## Basic Usage

```rust
let client = Client::new("…").await?;

let mut sub = client
    .block(<block_id>)          // build a block helper if you need context
    .events()                   // optional view onto events
    .set_retry_on_error(Some(true));

let mut blocks = client.sub(); // pseudo-code: see examples below
```

### Block Stream

```rust
let mut sub = client.block("latest").events();

loop {
    let events = sub.block().await?; // Vec<BlockPhaseEvent>
    handle(events);
}
```

### Transaction Stream

```rust
use avail_sdk::subscription::{TransactionSub, BlockExtOptionsSimple};

let opts = BlockExtOptionsSimple { filter: None, ..Default::default() };
let mut sub = TransactionSub::<MyCall>::new(client.clone(), opts);

sub.use_best_block(true);      // optional: follow best chain
sub.set_pool_rate(Duration::from_secs(2));

while let Ok((txs, info)) = sub.next().await {
    process_transactions(info.height, &txs);
}
```

## Configuration Knobs

- `use_best_block(bool)` – follow best (non-finalized) chain; default is
  finalized. Once initialized, switching modes has no effect.
- `set_block_height(u32)` – start iteration from a specific block. For
  replays, set this before the first `next()` call.
- `set_pool_rate(Duration)` – control polling cadence when waiting for new
  blocks.
- `set_retry_on_error(Option<bool>)` – override retry behaviour for RPC calls.

Subscriptions skip empty results automatically; they loop until data exists or
an RPC error occurs. When an error surfaces, the internal cursor rewinds so the
next call reattempts the failed block.

## Common Pitfalls

- **Runtime APIs missing** – `TransactionSub`/`ExtrinsicSub` require
  `system_fetch_extrinsics_v1`; without it, `next()` returns `Err`. Older blocks
  on archive nodes may hit this.
- **Unsigned extrinsics** – `TransactionSub` decodes signed extrinsics; if a
  block contains unsigned extrinsics, decoding fails and the error propagates.
- **Re-orgs** – when `use_best_block(true)` is set, a later re-org may reintroduce
  blocks you already processed. Track hashes to deduplicate.
- **Backpressure** – high polling rates against hosted RPC endpoints may trigger
  rate limits; adjust `set_pool_rate` accordingly.
- **Panics** – conversions inside subscription helpers follow the same rules as
  the underlying APIs (e.g., malformed account IDs panic). Validate strings up
  front.

## Stopping a Subscription

Subscriptions are simple async loops; dropping the struct cancels the in-flight
future. Wrap them in a task and use channels or cancellation tokens to stop them
gracefully.

## Testing Tips

- Use the mock client (`clients::mock_client`) to script block streams and error
  sequences.
- Disable retries (`set_retry_on_error(Some(false))`) to surface failures
  immediately in unit tests.
- When asserting behaviour across re-orgs, simulate best vs finalized modes
  explicitly.

Subscriptions are powerful building blocks for indexers, services that react to
on-chain events, and any workflow that needs to monitor inclusion without
hand-writing polling logic.
