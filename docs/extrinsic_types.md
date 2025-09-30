# Extrinsic Types in the SDK

Avail extrinsics can be viewed at different levels of abstraction depending on
your use case. The SDK exposes three primary representations:

1. **Raw extrinsics** – encoded byte payloads plus metadata (`BlockRawExtrinsic`).
2. **Decoded extrinsics** – call payloads with optional signature (`BlockExtrinsic<T>`).
3. **Signed transactions** – decoded extrinsics guaranteed to carry a signature
   (`BlockTransaction<T>`).

Understanding the differences helps you pick the correct view when traversing
blocks or analysing transactions.

## 1. Raw Extrinsics (`BlockRawExtrinsic`)

Produced by `BlockWithRawExt` (`client.block(id).raw_ext()`):

```rust
let mut raw_view = block.raw_ext();
let raw = raw_view.get(0u32, EncodeSelector::Extrinsic).await?;
```

Fields:

- `data: Option<String>` – hex-encoded extrinsic payload (SCALE bytes). When
  `EncodeSelector::None` is used, this will be `None`.
- `metadata: BlockExtrinsicMetadata` – includes block id, extrinsic index,
  pallet id, variant id, and block hash/number used to fetch the extrinsic.
- `signer_payload: Option<SignerPayload>` – contains `ss58_address`, `nonce`,
  `app_id`, and other signing data if the node exposes it.

Operations available on the raw view:

- Retrieve a specific extrinsic by index/hash via `get`.
- Retrieve the first/last extrinsic matching filters via `first`/`last`.
- Retrieve the complete set of matching extrinsics via `all`.
- Check existence or count without fetching payloads via `exists`/`count`.

Common use cases:

- Re-encoding or re-submitting raw extrinsics.
- Inspecting signer payload fields without decoding the entire call.
- Counting/existence checks by omitting payload bytes (`EncodeSelector::None`).

Conversion helpers:

- `BlockExtrinsic<T>::try_from(BlockRawExtrinsic)` – decode into a typed call
  (`T: HasHeader + Decode`).
- `BlockTransaction<T>::try_from(BlockRawExtrinsic)` – decode into a signed
  transaction (fails with `Err(String)` if the extrinsic lacks a signature).

## 2. Decoded Extrinsics (`BlockExtrinsic<T>`)

Produced by `BlockWithExt` (`client.block(id).ext()`):

```rust
let mut ext_view = block.ext();
let maybe_signed = ext_view.first::<avail::balances::calls::TransferKeepAlive>(Default::default()).await?;
```

Fields and helpers:

- `signature: Option<ExtrinsicSignature>` – present when the extrinsic is signed.
- `call: T` – decoded call struct (requires `T: HasHeader + Decode`).
- `metadata: BlockExtrinsicMetadata` – same metadata as the raw view.
- `events(&Client)` – fetch events tied to this extrinsic.
- `app_id()`, `nonce()`, `tip()`, `ss58_address()` – convenience accessors when a
  signature exists.

Use cases:

- Inspecting call parameters without caring whether the extrinsic was signed.
- Supporting pallets that emit unsigned extrinsics (e.g., inherent system
  extrinsics).
- Converting into `BlockTransaction<T>` when a signature is required.

## 3. Signed Transactions (`BlockTransaction<T>`)

Produced by `BlockWithTx` (`client.block(id).tx()`):

```rust
let mut tx_view = block.tx();
let signed = tx_view.first::<avail::balances::calls::TransferKeepAlive>(Default::default()).await?;
```

Guarantees and helpers:

- Always contains `signature: ExtrinsicSignature`; unsigned extrinsics produce
  an error during conversion.
- Provides the same helpers as `BlockExtrinsic<T>` plus `signature` is non-`None`.
- Exposes events via `events(&Client)` for convenience.

Use cases:

- Auditing account activity (nonce, tip, signer address).
- Checking transaction success/failure via events.
- Feeding signed extrinsics into analytics or re-broadcast pipelines.

## Choosing the Right Representation

| Requirement                               | Recommended View               |
|-------------------------------------------|--------------------------------|
| Need raw SCALE bytes or signer payload    | `BlockWithRawExt`              |
| Need to decode call parameters            | `BlockWithExt`                 |
| Need guaranteed signature metadata        | `BlockWithTx`                  |
| Need to check inclusion events            | `BlockWithExt` or `BlockWithTx`|
| Need lightweight existence/count checks   | `BlockWithRawExt` (`EncodeSelector::None`)

Illustrative transitions:

- Raw → Decoded call:

```rust
use avail_sdk::block_api::BlockExtrinsic;
use avail_sdk::avail;

let raw = block.raw_ext().get(0u32, EncodeSelector::Extrinsic).await?;
if let Some(raw) = raw {
  let decoded: BlockExtrinsic<avail::balances::calls::TransferKeepAlive> = raw.try_into()?;
      println!("call args: {:?}", decoded.call.value);
}
```

- Decoded → Signed transaction:

```rust
use avail_sdk::block_api::BlockTransaction;

let decoded = block.ext().first::<avail::balances::calls::TransferKeepAlive>(Default::default()).await?;
if let Some(decoded) = decoded {
    let signed: BlockTransaction<_> = decoded.try_into()?; // fails if unsigned
    println!("nonce: {}", signed.nonce());
}
```

You can transition between representations:

- `BlockWithRawExt` → `BlockWithExt` / `BlockWithTx` via the view builders.
- `BlockExtrinsic<T>` → `BlockTransaction<T>` using `BlockTransaction::try_from`
  (fails when unsigned).

Understanding these layers makes it easier to balance performance (raw view)
against ergonomics (decoded view) and accuracy (signed transactions).
