# Interface Failure Modes

This SDK talks to an Avail node exclusively through RPC calls. As a result, most
operations share the same broad failure categories:

- **Transport or endpoint issues** – networking hiccups, timeouts, TLS errors,
  or the node being offline. These bubble up as `RpcError` (for raw RPC calls)
  or wrapped inside `Error` where we combine multiple causes.
- **Missing runtime support** – older runtimes may not expose a given RPC or
  runtime API (e.g., `system_fetch_events_v1`). In those cases the node returns
  `Unsupported`/`CallError` and we surface it directly.
- **Decoding errors** – when converting hashes, account ids, or SCALE-encoded
  payloads fails. These appear as `UserError::Decoding` or
  `RpcError::DecodingFailed` depending on the context.
- **Unexpected `None` responses** – some helpers expect data to exist. If the
  node replies with `None`, we either translate it to
  `RpcError::ExpectedData(...)` or return `Ok(None)`; check the method docs to
  see which behavior applies.
- **Caller input issues** – malformed strings, invalid indices, or providing an
  unsigned extrinsic where a signature is required. These produce
  `UserError::Other` variants or panic when noted in the docs (e.g., conversion
  helpers that declare `# Panics`).

Below is a quick reference grouped by interface.

## Client Construction (`Client`, `OnlineClient`)

- `Client::new`, `Client::from_rpc_client`, `OnlineClient::new` – fail if the
  endpoint cannot be reached, metadata decoding fails, or fetching runtime
  version/genesis hash errors out.
- Methods that return metadata, spec version, etc., only panic if the internal
  `RwLock` is poisoned (should never happen unless the process panicked during a
  write).

## Chain RPC (`Client::chain()` / `ChainApi`)

- `block_hash`, `block_header`, `legacy_block` – network failures, invalid hash
  conversions, or the node returning `None`. When `retry_on_none` is `false`,
  `None` is returned; otherwise the call retries until data is available.
- `account_info`, `account_nonce`, `account_balance` – invalid account id
  strings, missing storage, or RPC failures.
- `block_state` – invalid block identifier, missing block hash, or RPC errors.
- `system_fetch_extrinsics`, `system_fetch_events` – runtime API not exposed or
  RPC failures.
- `grandpa_block_justification*` – node not running the GRANDPA RPC extension or
  failure decoding the returned justification.

## Block Helpers (`BlockApi`)

- All views (`tx`, `ext`, `raw_ext`, `events`) ultimately depend on
  `ChainApi`; they fail for the same reasons plus:
  - Filtering by index/hash that does not exist returns `Ok(None)`.
  - Decoding payloads into a target type can fail with
    `UserError::Decoding` or `RpcError::ExpectedData` when the node omits
    bytes.
  - Signing-specific helpers (`BlockWithTx`) error when the extrinsic is
    unsigned.

## Subscriptions (`Sub`, `BlockEventsSub`, etc.)

- Underlying errors come from `ChainApi` (network, decoding). When a pull fails
  we rewind the internal cursor and propagate the error so callers can retry.
- Subscriptions skip empty results; if the node continuously returns empty data
  the call will loop until something appears or an RPC error is raised.

## Transaction Builders (`TransactionApi`, `Options`) 

- `Options::build` – fails when fetching account nonce or finalized header fails
  (e.g., endpoint unreachable), or when mortality refinement cannot resolve a
  block hash.
- `TransactionApi` methods panic when a supplied `AccountIdLike/MultiAddress`
  string cannot be converted; this is highlighted in the docs. RPC submission
  errors from `sign_and_submit` surface as `Error`.

## `OnlineClient`

- Only fails during construction (`OnlineClient::new`) when fetching metadata,
  runtime version, or genesis hash fails, or when decoding that data hits an
  error. The getters/setters themselves are infallible besides lock poisoning.

If you need to differentiate between transport failures and business logic
errors, inspect the concrete variant of `Error`/`RpcError` returned by these
methods. Many helpers also provide retry toggles (`set_retry_on_error` or
`retry_on(...)`) so you can decide whether transient failures should bubble up
immediately.
