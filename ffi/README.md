# Avail Rust FFI Playground

This little crate wraps the `avail-rust` client so you can poke at Avail from C
or Perl without having to understand the whole Rust codebase first. The
standalone examples under `c/` and `perl/` are kept runnable and have been
smoke-tested after the latest changes.

## What the FFI exposes

The Rust side lifts the following symbols for foreign callers:

- `hello_from_rust()` – quick connectivity check.
- `initialize_client(endpoint)` – spins up a client bound to an RPC endpoint.
- `initialize_signer(seed)` – returns a signer handle created from a seed phrase.
- `do_submit_data(signer_handle, data, app_id)` – signs and submits a data
  availability transaction.
- `get_transaction_receipt(submitted_tx_handle)` – fetches a receipt handle.
- `receipt_new(receipt_handle)` / `receipt_free(receipt_ptr)` – manage the
  heap-allocated receipt wrapper.
- `receipt_block_height(receipt_ptr)` / `receipt_transaction_index(receipt_ptr)`
  – accessors for basic metadata.
- `receipt_block_hash(receipt_ptr)` / `receipt_transaction_hash(receipt_ptr)` –
  raw pointers to the 32-byte hashes (callers own the formatting).

## Building the library

```
cargo build
```

The shared object ends up in `target/debug` (or `target/release` if you flip
profiles). Both examples look in `../target/debug`, so tweak `libpath` if you
switch profiles.

## C example (manually tested)

- Entry point: `c/main.c`
- Helper script: `run_c.sh`

Run `./run_c.sh` to rebuild the crate if needed and execute the example. On the
latest run it printed the receipt details plus hex-encoded transaction and block
hashes.

## Perl example (manually tested)

- Script: `perl/main.pl`
- Requires: `FFI::Platypus`, `FFI::Platypus::Buffer`, `FFI::CheckLib`

Invoke with `perl perl/main.pl`. It attaches to the Rust library, submits a
message, then displays block height, transaction index, and the two hashes as
hex strings. This was exercised alongside the C run to confirm the shared code
paths behave.

## Housekeeping tips

- Always call `receipt_free` once you are done with a receipt pointer.
- If you rebuild in release mode, copy or symlink the `.so` (or `.dylib`/`.dll`)
  where your host language expects it.
- When extending the FFI, keep the examples up to date—they double as living
  tests.
