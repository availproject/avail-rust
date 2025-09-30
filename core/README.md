# Avail Rust Core

**Avail Rust Core provides the low-level building blocks for interacting with
Avail data-availability networks from Rust.**

This crate ships the strongly-typed runtime metadata, SCALE helpers, storage
abstractions, and RPC utilities that power the higher-level
[`avail-rust-client`](https://crates.io/crates/avail-rust-client) crate. Use it
when you want tight control over encoding/decoding Avail extrinsics, events, and
storage entries without pulling in a full client stack.

## What You Get
- Avail-specific runtime types (`AccountId`, `RuntimeCall`, pallet storage
  definitions, and more).
- Helpers to encode/decode extrinsics, events, and receipts via
  `TransactionDecodable`, `TransactionEventDecodable`, and related traits.
- Storage traits (`StorageValue`, `StorageMap`, `StorageDoubleMap`) with
  iterators and hashing helpers to fetch and decode on-chain data.
- Lightweight RPC helpers built on `subxt`, including tools for fetching raw
  blocks, events, and extrinsics.
- Dual-target support: `native` (std) and `wasm` feature flags expose the same
  API with the right transport integrations for each environment.

## Feature Flags
- `native` *(default)*: Enables std support plus native RPC transports.
- `wasm`: Enables bindings required for WebAssembly environments.

Enable exactly one of these targets; the crate deliberately avoids picking a
transport for you so that it can stay lightweight.

## Related Crates
- [`avail-rust-client`](https://crates.io/crates/avail-rust-client): Batteries
  included client that wraps `avail-rust-core` with RPC transport, signing, and
  ergonomic submission helpers.
- [`avail-rust` examples](https://github.com/availproject/avail-rust/tree/main/examples):
  Real-world programs that showcase how the core types are meant to be used.

