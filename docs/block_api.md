# Block API Guide

The `block` module provides convenient views for inspecting block data,
extrinsics, and emitted events. This guide summarizes the available helpers and
how they fit together.

## Creating a Block Handle

```rust
let client = Client::new("…").await?;
let block = client.block(hash_or_height);
```

`hash_or_height` can be any type convertible into `HashStringNumber` (e.g., a
hex hash string, `H256`, or block number). No RPC calls happen yet; data is
retrieved lazily.

### Retry Behaviour

`Block` inherits the client's global retry flag and exposes
`set_retry_on_error(Option<bool>)` so you can override it per handle:

```rust
let mut block = client.block(hash);
block.set_retry_on_error(Some(false)); // disable retries for this handle
```

## Views

`Block` acts as a factory for specialized views:

- `block.signed()` → [`SignedExtrinsics`](#signedextrinsics) (signed extrinsics
  only)
- `block.ext()` → [`Extrinsics`](#extrinsics) (decoded extrinsics with optional
  signatures)
- `block.encoded()` → [`EncodedExtrinsics`](#encodedextrinsics) (raw payloads +
  metadata)
- `block.events()` → [`Events`](#events) (block-wide events)

Each view shares the same `block_id` and retry configuration. You can clone the
view to inspect different filters in parallel without reinitializing the base
`Block` handle.

### EncodedExtrinsics

Works with raw extrinsics; useful when you need the SCALE bytes, signer payload,
or when decoding into a specific call type is too expensive.

```rust
let mut raw = block.encoded();
raw.set_retry_on_error(Some(true));

if let Some(ext) = raw.get(0u32, EncodeSelector::Extrinsic).await? {
    println!("hash: {:?}", ext.ext_hash());
}

let all = raw.all(ExtrinsicsOpts::default()).await?; // Vec<EncodedExtrinsic>
```

Common operations:

- `get(id, encode_as)` – fetch a single extrinsic by index/hash.
- `first(opts)`, `last(opts)` – fetch first/last extrinsic matching filters.
- `all(opts)` – fetch all matching extrinsics.
- `count(opts)`, `exists(opts)` – inspect matches without transferring payloads.

`ExtrinsicsOpts` lets you filter by:

- `filter: ExtrinsicFilter` – pallet/variant index, extrinsic position, or hash.
- `ss58_address` – only extrinsics signed by a specific account (requires
  signer payload support).
- `app_id` / `nonce` – restrict to extrinsics with matching signer payload data.
- `encode_as: EncodeSelector` – choose the encoding (full extrinsic, call
  payload only, none, etc.).

When `encode_as = EncodeSelector::None`, the node omits payload bytes, which is
handy for counting/existence checks.

### Extrinsics

Wraps `EncodedExtrinsics` but decodes extrinsic payloads into target types.

```rust
let balance = block
    .ext()
    .first::<avail::balances::calls::TransferAllowDeath>(ExtrinsicsOpts::default())
    .await?;

if let Some(extrinsic) = balance {
    println!("tip: {:?}", extrinsic.tip());
}
```

- `get`, `first`, `last`, `all` mirror the encoded view but return
  `Extrinsic<T>` where `T: HasHeader + Decode`. `HasHeader::HEADER_INDEX` is used
  to infer the pallet/function indices when filters are absent.
- `count`, `exists` work without decoding payloads (using header information),
  which is useful for lightweight checks.

`Extrinsic<T>` exposes helpers:

- `signature: Option<ExtrinsicSignature>` – raw signature data.
- `call: T` – decoded call struct.
- `metadata: Metadata` – includes block id, extrinsic index, pallet and variant
  ids.
- `events(&Client)` – fetch events tied to this extrinsic.
- `app_id()`, `nonce()`, `tip()`, `ss58_address()` – convenience accessors when a
  signature exists.

### SignedExtrinsics

Restricts results to extrinsics with signatures and wraps them as
`SignedExtrinsic<T>`.

```rust
let signed = block
    .signed()
    .first::<avail::balances::calls::TransferAllowDeath>(ExtrinsicsOpts::default())
    .await?;

if let Some(tx) = signed {
    println!("signer nonce: {:?}", tx.nonce());
}
```

This view is ideal when you need to inspect signer metadata (nonce, tip,
origin). Internally it reuses `Extrinsics` and filters out unsigned results.

### Events

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

`EventsOpts` lets you filter by extrinsic index and choose whether the node
should encode/decode event data.

## Metadata Structures

- `Metadata` – pallet/index metadata for a single extrinsic.
- `EncodedExtrinsic` – raw data + metadata + optional signer payload.
- `Extrinsic<T>` – decoded extrinsic call and optional signature.
- `SignedExtrinsic<T>` – guaranteed signed extrinsic, with helpers for nonce,
  tip, and event lookups.
- `ExtrinsicEvents` – wrapper over `Vec<ExtrinsicEvent>` with convenience
  methods (`first`, `all`, `is_extrinsic_success_present`, etc.).

## Error Handling

- Missing extrinsics or events surface as `Ok(None)` or
  `Err(RpcError::ExpectedData)` depending on the operation.
- Decoding failures produce `UserError::Decoding` (e.g., header mismatch) or
  `RpcError::DecodingFailed`.
- Underlying RPC failures propagate as `Error::RpcError`.
- Missing runtime support (no `system_fetch_extrinsics_v1`) surfaces as
  `RpcError::CallError` with `Unsupported`.
- When `EncodeSelector::None` is used, accessing `EncodedExtrinsic::data` yields
  `None`; ensure you check for missing payloads before decoding.

## When to Use the Block Helpers

Use `Block` when you need rich block inspection beyond raw storage queries:

- Indexers retrieving extrinsic payloads or signer metadata.
- Tools validating events emitted by specific calls.
- Debugging transaction inclusion (via `SignedExtrinsics` + `ExtrinsicEvents`).

For event-only workflows, `BlockSub` or `Events` subscriptions might be simpler;
for raw storage reads, use the storage APIs instead.

### Example: Filtering by Pallet/Variant

```rust
use avail_sdk::block::ExtrinsicsOpts;
use avail_sdk::avail;

let mut opts = ExtrinsicsOpts::default();
opts.filter = Some((avail::balances::PALLET_ID, avail::balances::calls::transfer_allow_death::FUNCTION_ID).into());

let mut raw = block.encoded();
let all_balances_calls = raw.all(opts).await?;
```

### Example: Fetching by Hash

```rust
let raw = block.encoded();
let extrinsic = raw
    .get("0x…extrinsic_hash", EncodeSelector::Extrinsic)
    .await?;

if let Some(ext) = extrinsic {
    println!("Signer: {:?}", ext.ss58_address());
}
```
