# Basic Extrinsic Submission

This page walks through submitting an extrinsic using the SDK and highlights common
pitfalls you might encounter in production.

## End-to-End Flow

1. **Create a client**

   ```rust
   use avail_sdk::client::Client;

   let client = Client::new("https://rpc-mainnet.avail.so").await?;
   ```

   *Pitfalls*
   - Endpoint unreachable, TLS misconfiguration, wrong protocol (HTTP vs HTTPS) → `Client::new` returns `Err(Error)`.
   - Outdated runtime metadata, node running incompatible runtime version → `OnlineClient::new` returns `RpcError::DecodingFailed`.
   - DNS lookup or proxy failures surface in the same `Error` as transport failures.
   - If retries are disabled the first transient timeout bubbles up immediately; enable `retry` in `Client::new_ext` for better resilience.

2. **Build transaction options**

   ```rust
   use avail_sdk::transaction_options::Options;

   let opts = Options::default()
       .app_id(0)
       .mortality(Options::default().mortality.unwrap_or_default())
       .tip(0);

   let signer = avail_sdk::subxt_signer::sr25519::Keypair::from_string("<seed>", None)?;
   let account_id = signer.public_key().to_account_id();
   let refined = opts.build(&client, &account_id, client.is_global_retries_enabled()).await?;
   ```

   *Pitfalls*
   - Missing nonce when the node is unreachable → `Options::build` cannot fetch `account_nonce`.
   - Mortality period too short → transaction may expire before inclusion; too long (e.g., 65,536) may exceed runtime limits.
   - Using an `app_id` that doesn’t exist in the runtime → extrinsic fails at dispatch.
   - Off-chain clocks drifting heavily from the network → mortality anchor (finalized block) may lag behind expectations.
   - Concurrent submissions reusing the same builder (with `nonce(None)`) can still race; prefer locking or explicit nonce management.

3. **Compose the call**

   ```rust
   let call = client
       .tx()
       .balances()
       .transfer_keep_alive("5F...", 1_000_000_000);
   ```

   *Pitfalls*
   - Address conversion panics if the string is malformed (documented as `# Panics`).
   - Passing an unsupported multi-address type (e.g., an enum variant disabled in the runtime) panics.
   - The call itself may require specific origins; e.g., calling `balances::transfer_all` from a proxy account without the right proxy type will fail later during execution.

4. **Sign and submit**

   ```rust
   let submitted = client
       .sign_and_submit_call(&signer, &call.call, Options::default())
       .await?;
   println!("Submitted hash: {}", submitted.tx_hash);
   ```

   *Pitfalls*
   - Nonce race → if you run multiple submissions concurrently without explicit nonces, the node may reject with “Invalid Transaction: Future” or “Stale”.
   - Node rejects unsigned extrinsic → ensure the call expects a signed origin.
   - Tip/fee insufficient → submission returns `RpcError::CallError` with `InvalidTransaction::Payment`.
   - Transaction larger than `max_extrinsic_size` → `RpcError::CallError` (`InvalidTransaction::ExhaustsResources`).
   - Runtime upgrade mid-flight → `spec_version` mismatch; the node rejects until you refresh metadata and retry.
   - Using an offline signer with outdated nonce/mortality values results in `InvalidTransaction` errors from the node.

5. **Track inclusion**

   ```rust
   let receipt = submitted.receipt(false).await?; // `false` = follow finalized chain
   match receipt {
       Some(r) => println!("Included in block {}", r.block_ref.height),
       None => println!("Still pending or dropped"),
   }
   ```

   *Pitfalls*
   - Mortality expired → `receipt` eventually returns `None`.
   - Following best chain (`use_best_block = true`) can report inclusion that later gets re-orged out; use finalized for certainty.
   - RPC archive gaps (pruned node) → `.receipt()` fails with `Error` even though the transaction was once included.
   - If the runtime lacks `system_fetch_extrinsics_v1`, event/extrinsic lookups fail; monitor finality via events or account nonce instead.

## Additional Gotchas

- **Retry settings** – `Client::set_global_retries_enabled(false)` means helpers propagate network failures immediately. Enable retries in production to smooth transient outages.
- **Custom runtimes** – If the runtime doesn’t expose `system_fetch_extrinsics_v1`, block/extrinsic helpers will fail. Use raw storage queries as a fallback.
- **Metadata changes** – If the runtime upgrades (new spec version), cached metadata may become outdated. Restart the client or refresh via `OnlineClient::set_metadata`.
- **Signing key management** – Keep seed phrases secure. For multisig or proxy operations, ensure you sign extrinsics using the correct account.
- **Mortality and clock drift** – Submitting with very short mortality windows (e.g., period 4) across regions with poor latency can lead to frequent expirations.
- **Resource limits** – Exceeding block weight or length causes the runtime to drop the extrinsic; check weight in advance (via `transaction_payment_query_call_info`).
- **Node-specific limits** – Rate limits or quotas imposed by hosted RPC providers may throttle or reject requests; handle `RpcError::Transport` accordingly.

Following the steps above should get you from a keypair and call to an on-chain
transaction while understanding the main failure points along the way.
