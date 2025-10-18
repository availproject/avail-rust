# Retry Mechanism Overview

Many helpers in the SDK can retry RPC calls automatically. This document explains
how retry behaviour is configured and which entry points respect it.

## Global Toggle

The global flag lives on `OnlineClient` (and by extension `Client`):

```rust
let client = Client::new("…").await?;
client.set_global_retries_enabled(true); // default
```

- `true` (default) – retry eligible RPC calls on transient failures.
- `false` – propagate the first error immediately.

When the flag is `false`, you can still opt-in per-call using `.retry_on(...)`
builders described below.

## Chain API (`Client::chain()`)

`Chain` exposes fine-grained control:

```rust
let chain = client.chain().retry_on(Some(true), Some(true));
```

- First parameter (`error`) toggles retries on transport errors.
- Second parameter (`none`) toggles retries when the RPC returns `None` (e.g.,
  missing block hash). Default is `false` to avoid spinning indefinitely.

When neither override is provided, the API inherits the global flag for errors
and assumes `retry_on_none = false`.

## Other Helpers

Most higher-level builders delegate to `Chain` and therefore inherit its
behaviour:

- `Block`, `EncodedExtrinsics`/`Extrinsics`/`SignedExtrinsics`, `Events`
- Subscriptions (`Sub`, `BlockEventsSub`, etc.)
- Transaction submission (`sign_and_submit_call`, `Options::build()`)

These helpers typically expose a `set_retry_on_error(Option<bool>)` method. The
rules are:

1. `Some(true)` – force retries regardless of global setting.
2. `Some(false)` – disable retries even if global setting is `true`.
3. `None` – inherit the client/global preference.

Example:

```rust
let mut block = client.block(hash);
block.set_retry_on_error(Some(false)); // read block data without retries
```

## What Gets Retried?

Retries wrap individual RPC calls (HTTP requests to the node). Failures that
qualify:

- Network errors: timeout, connection reset, temporary DNS failure.
- `RpcError::Transport` and some `JsonRpseeError` values.
- Optional `None` results when `retry_on_none = true`.

Failures that **do not** get retried automatically:

- Deterministic runtime errors (e.g., `InvalidTransaction` during submission).
- Decoding errors caused by malformed data (`RpcError::DecodingFailed`).
- Panics triggered by invalid user input (e.g., bad address strings).
- Runtime API missing or unsupported (`RpcError::CallError` with `Unsupported`).

Retries use exponential backoff (implemented in `clients::utils`) and respect
async cancellation if you drop the future.

## Per-call Overrides

Some method chains allow overriding the retry flag just before the RPC call:

```rust
client
    .chain()
    .retry_on(Some(true), None)
    .block_hash(Some(height))
    .await?;
```

You can combine this with the global flag to enforce different policies across
components (e.g., enable retries for chain queries but disable them for metadata
fetches).

## When to Disable Retries

- During tests where deterministic behaviour is required.
- When interacting with a node that signals backpressure via errors (avoid
  hammering it with automatic retries).
- If you prefer to implement custom retry logic (e.g., with application-level
  metrics or circuit breakers).

Otherwise, keep retries enabled—they significantly reduce transient failures in
production environments.
