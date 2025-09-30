# Error Handling Guide

Interacting with an Avail node involves several error types. Understanding the
distinction helps you design resilient applications.

## Error Types

| Type               | Description                                                             |
|--------------------|-------------------------------------------------------------------------|
| `RpcError`         | Raw errors originating from the RPC layer (transport, call, decoding).  |
| `Error`            | High-level wrapper used across the SDK combining `RpcError`, user input
|                    | issues, and other context-specific failures.                            |
| `UserError`        | Indicates caller-side problems such as invalid conversion or validation |
|                    | failures (e.g., malformed address strings).                             |

Most SDK functions return either `Result<T, Error>` or `Result<T, RpcError>`. You
can inspect or downcast them as follows:

```rust
match client.chain().block_hash(Some(height)).await {
    Ok(Some(hash)) => { /* use hash */ }
    Ok(None) => { /* block not found */ }
    Err(e) => match e {
        RpcError::Transport(msg) => retry(msg),
        RpcError::CallError { error, .. } => handle_runtime_error(error),
        other => log::error!("unexpected error: {other:?}"),
    },
}
```

## Common Variants

- `RpcError::Transport` – network/TLS issues; typically transient.
- `RpcError::CallError` – runtime rejected the call (e.g., `InvalidTransaction`).
- `RpcError::DecodingFailed` – runtime metadata mismatch or malformed response.
- `RpcError::ExpectedData` – node returned `None` when data was required.
- `UserError::Decoding` – failed to parse a `HashStringNumber`, address, etc.
- `UserError::ValidationFailed` – pre-flight validation failed (e.g., invalid
  parameter ranges).

## Pattern Matching on `Error`

`Error` wraps multiple sources. Use `Error::source()` or custom `match` arms:

```rust
use avail_sdk::Error;

fn classify(err: Error) {
    if let Some(rpc) = err.rpc_error() {
        println!("rpc error: {rpc:?}");
    } else if let Some(user) = err.user_error() {
        println!("user error: {user:?}");
    } else {
        println!("other error: {err:?}");
    }
}
```

Check the convenience methods on `Error` (e.g., `is_transport_error()`) if
available in your version.

## Retry Decision

- Automatic retries are controlled by `Client::set_global_retries_enabled` or
  `.retry_on(...)`. When a retryable error occurs the helper will attempt again
  before surfacing `Err`.
- For non-retryable errors (invalid extrinsic, decoding), the SDK returns `Err`
  immediately.

## Recommended Handling Patterns

- **Submission pipeline** – separate “build → sign → submit” errors (likely user
  or runtime issues) from “track receipt” errors (likely network/endpoint).
- **Subscriptions** – treat any `Err(Error)` from `.next()` as an opportunity to
  log and decide whether to restart the subscription or abort.
- **Decoding** – wrap conversions (addresses, hashes) in helper functions that
  validate before passing them to the SDK to avoid panics.

## Mapping to Application Errors

If your application has its own error enum, consider implementing `From<Error>`
and `From<RpcError>` to normalise handling. Example:

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("network failure: {0}")]
    Network(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("runtime error: {0}")]
    Runtime(String),
}

impl From<RpcError> for AppError {
    fn from(err: RpcError) -> Self {
        match err {
            RpcError::Transport(e) => AppError::Network(e),
            RpcError::CallError { error, .. } => AppError::Runtime(format!("{error:?}")),
            other => AppError::Runtime(format!("{other:?}")),
        }
    }
}

impl From<Error> for AppError {
    fn from(err: Error) -> Self {
        if let Some(rpc) = err.rpc_error() {
            return AppError::from(rpc.clone());
        }
        if let Some(user) = err.user_error() {
            return AppError::InvalidInput(format!("{user:?}"));
        }
        AppError::Runtime(format!("{err:?}"))
    }
}
```

By recognising which errors are under your control (inputs, options) vs those
coming from the node (network, runtime), you can build better recovery and
observability into your application.
