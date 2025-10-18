# SubmittedTransaction::receipt Accuracy

`SubmittedTransaction::receipt(use_best_block)` walks the chain looking for the
extrinsic you previously signed and submitted. The helper is reliable **when the
following assumptions hold**:

1. **The transaction options reflect the actual submission** – the
   `SubmittedTransaction` stores the nonce, mortality window, and account id that
   were used when signing. If you mutate those values before calling
   `.receipt()`, or if the transaction was rebroadcast with different options,
   the lookup may return `Ok(None)` even though a different version of the
   transaction was included.

2. **The runtime exposes `system_fetch_extrinsics_v1`** – `receipt()` ultimately
   calls `Block::encoded().get` which depends on this runtime API. Older
   runtimes or archive nodes without the extension will surface `Error`.

3. **The transaction lands within its mortality window** – the search starts at
   `mortality.block_height` and scans until `mortality.block_height + period`.
   If the transaction expires or is included beyond that range (which should be
   impossible on a correct node), the helper returns `Ok(None)`.

4. **The node keeps serving the relevant block** – if the RPC endpoint prunes
   older blocks or refuses to decode extrinsics for them, the helper fails with
   `Error` even if the transaction was once included.

5. **You query the desired fork** – `use_best_block = false` follows the
   finalized chain. Once it returns `Some`, the result is stable (barring bugs in
   the finality gadget). `use_best_block = true` follows the best (non-finalized)
   chain; a later re-org can invalidate the block you saw earlier.

6. **No long-term network failures** – `.receipt()` performs several RPC calls in
   sequence (block subscription, account nonce lookup, fetching raw extrinsics).
   Any transport failure surfaces as `Err(Error)`.

Under these conditions, `.receipt(false)` will eventually return one of two
states:

- `Ok(Some(TransactionReceipt))` once the extrinsic is finalized and the block is
  still available.
- `Ok(None)` when the transaction never made it on chain (dropped, replaced, or
  expired).

`.receipt(true)` provides faster feedback by following the best chain, but the
result is only as durable as that fork: if the block is later re-orged out, a
subsequent call may return `Ok(None)` even though an earlier call returned data.

If any of the assumptions above cannot be guaranteed in your environment, avoid
relying solely on `.receipt()` or wrap it with additional checks (e.g., monitor
events, query account nonce directly, or follow finalized blocks only).
