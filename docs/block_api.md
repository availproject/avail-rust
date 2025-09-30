# Block API Guide

The `BlockApi` module provides convenient views for inspecting block data,
extrinsics, and emitted events. This guide summarizes the main building blocks
and how they interrelate.

## Creating a Block Handle

```rust
let client = Client::new("…").await?;
let block = client.block(hash_or_height);
```

`hash_or_height` can be any type convertible into `HashStringNumber` (e.g., a
hex hash string, `H256`, or block number). No RPC calls happen yet; data is
retrieved lazily.

### Retry Behaviour

`BlockApi` inherits the global retry flag and exposes
`set_retry_on_error(Option<bool>)` so you can override it per handle:

```rust
let mut block = client.block(hash);
block.set_retry_on_error(Some(false)); // disable retries for this handle
```

## Views

`BlockApi` acts as a factory for specialized views:

- `block.tx()` → [`BlockWithTx`](#blockwithtx) (signed transactions only)
- `block.ext()` → [`BlockWithExt`](#blockwithext) (decoded extrinsics with
  optional signature)
- `block.raw_ext()` → [`BlockWithRawExt`](#blockwithrawext) (raw encoded payloads
  + metadata)
- `block.events()` → [`BlockEvents`](#blockevents) (block-wide events)

Each view shares the same `block_id` and retry configuration. You can clone the
view to inspect different filters in parallel without reinitializing the base
`BlockApi`.

### BlockWithRawExt

Works with raw extrinsics; useful when you need the SCALE bytes, signer payload,
or when decoding into a specific call type is too expensive.

```rust
let mut raw = block.raw_ext();
raw.set_retry_on_error(Some(true));

if let Some(ext) = raw.get(0u32, EncodeSelector::Extrinsic).await? {
    println!("hash: {:?}", ext.ext_hash());
}

let all = raw.all(Default::default()).await?; // Vec<BlockRawExtrinsic>
```

Common operations:

- `get(id, encode_as)` – fetch a single extrinsic by index/hash.
- `first(opts)`, `last(opts)` – fetch first/last extrinsic matching filters.
- `all(opts)` – fetch all matching extrinsics.
- `count(opts)`, `exists(opts)` – inspect matches without transferring payloads.

`BlockExtOptionsExpanded` lets you filter by:

- `filter: ExtrinsicFilter` – pallet/variant index, extrinsic position, or hash.
- `ss58_address` – only extrinsics signed by a specific account (requires
  signer payload).
- `app_id` / `nonce` – restrict to extrinsics with matching signer payload data.
- `encode_as: EncodeSelector` – choose the encoding (full extrinsic, call
  payload only, etc.).

When `encode_as = EncodeSelector::None`, the node omits payload bytes, which is
handy for counting/existence checks.

### BlockWithExt

Wraps `BlockWithRawExt` but decodes extrinsic payloads into target types.

```rust
let balance = block
    .ext()
    .first::<avail::balances::calls::TransferAllowDeath>(Default::default())
    .await?;

if let Some(extrinsic) = balance {
    println!("tip: {:?}", extrinsic.tip());
}
```

- `get`, `first`, `last`, `all` mirror the raw view but return `BlockExtrinsic<T>`
- `get`, `first`, `last`, `all` mirror the raw view but return `BlockExtrinsic<T>`
  where `T: HasHeader + Decode`. `HasHeader::HEADER_INDEX` is used to infer the
  pallet/function indices.
- `count`, `exists` work without decoding payloads (using header information),
  which is useful for lightweight checks.

`BlockExtrinsic<T>` exposes helpers:

- `signature: Option<ExtrinsicSignature>` – raw signature data.
- `call: T` – decoded call struct.
- `metadata: BlockExtrinsicMetadata` – includes block id, extrinsic index, pallet
  and variant ids.
- `events(&Client)` – fetch events tied to this extrinsic.
- `app_id()`, `nonce()`, `tip()`, `ss58_address()` – convenience accessors when a
  signature exists.

### BlockWithTx

Restricts results to extrinsics with signatures and wraps them as
`BlockTransaction<T>`.

```rust
let signed = block
    .tx()
    .first::<avail::balances::calls::TransferAllowDeath>(Default::default())
    .await?;

if let Some(tx) = signed {
    println!("signer nonce: {:?}", tx.nonce());
}
```

This view is ideal when you need to inspect signer metadata (nonce, tip,
origin).

### BlockEvents

Fetches events for the entire block or for a specific extrinsic index.

```rust
let events = block.events().block(Default::default()).await?;

let tx_events = block.events().ext(3).await?;
if let Some(events) = tx_events {
    if events.is_extrinsic_success_present() {
        println!("Extrinsic 3 succeeded");
    }
}
```

`BlockEventsOptions` lets you filter by extrinsic index and choose whether the
node should encode/decode event data.

## Metadata Structures

- `BlockExtrinsicMetadata` – pallet/index metadata for a single extrinsic.
- `BlockRawExtrinsic` – raw data + metadata + optional signer payload.
- `BlockExtrinsic<T>` – decoded extrinsic call and optional signature.
- `BlockTransaction<T>` – guaranteed signed extrinsic, with helpers for nonce,
  tip, and event lookups.
- `ExtrinsicEvents` – wrapper over `Vec<ExtrinsicEvent>` with convenience
  methods (`first`, `all`, `is_extrinsic_success_present`, etc.).

## Error Handling

- Missing extrinsics or events surface as `Ok(None)` or
  `Err(RpcError::ExpectedData)` depending on the operation.
- Decoding failures produce `UserError::Decoding` (e.g., header mismatch) or
- Decoding failures produce `UserError::Decoding` (e.g., header mismatch) or
- Underlying RPC failures propagate as `Error::RpcError`.
- Underlying RPC failures propagate as `Error::RpcError`.
  `RpcError::DecodingFailed`.
- Missing runtime support (no `system_fetch_extrinsics_v1`) surfaces as
  `RpcError::CallError` with `Unsupported`.
- When `EncodeSelector::None` is used, accessing `BlockRawExtrinsic::data` yields
  `None`; ensure you check for missing payloads before decoding.
- Underlying RPC failures propagate as `Error::RpcError`.

## When to Use BlockApi

Use `BlockApi` when you need rich block inspection beyond raw storage queries:

- Indexers retrieving extrinsic payloads or signer metadata.
- Tools validating events emitted by specific calls.
- Debugging transaction inclusion (via `BlockWithTx` + `ExtrinsicEvents`).

For event-only workflows, `BlockEventsSub` might be simpler; for raw storage
reads, use the storage APIs instead.
### Example: Filtering by Pallet/Variant

```rust
use avail_sdk::block_api::{BlockExtOptionsExpanded, BlockApi};
use avail_sdk::avail;

let mut opts = BlockExtOptionsExpanded::default();
opts.filter = Some((avail::balances::PALLET_ID, avail::balances::calls::transfer_allow_death::FUNCTION_ID).into());

let mut raw = block.raw_ext();
let all_balances_calls = raw.all(opts).await?;
```

### Example: Fetching by Hash

```rust
let raw = block.raw_ext();
let extrinsic = raw
    .get("0x…extrinsic_hash", EncodeSelector::Extrinsic)
    .await?;
```

Both examples fall back to the same RPC call (`system_fetch_extrinsics_v1`) but
the helper handles filter construction and response parsing.
- `get`, `first`, `last`, `all` mirror the raw and decoded views but guarantee a
  signature (returning `BlockTransaction<T>`).
- `BlockTransaction<T>` adds additional helpers: `signature`, `call`, `metadata`,
  plus methods like `events(&Client)`, `app_id()`, `nonce()`, `tip()`, and
  `ss58_address()`.

`BlockTransaction::try_from(BlockExtrinsic)` is used internally; unsigned
extrinsics cause an error, ensuring you only get signed data.
Example:

```rust
let events_view = block.events();

// Fetch all events in the block
let all_events = events_view.block(Default::default()).await?;

// Fetch events emitted by extrinsic at index 5
if let Some(events) = events_view.ext(5).await? {
    if events.is_extrinsic_success_present() {
        println!("Extrinsic succeeded");
    }
}
```

`BlockEventsOptions` fields:

- `filter: Option<EventFilter>` – fetch events for a specific extrinsic index.
- `enable_encoding` / `enable_decoding` – tell the node whether to return raw
  bytes (`encoded_data`) and/or decoded fields (`data`). Leaving these as `None`
  uses the node defaults (encoding enabled, decoding disabled).
