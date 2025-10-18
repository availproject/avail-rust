# Extrinsic Types in the SDK

Avail extrinsics can be viewed at different levels of abstraction depending on
your use case. The SDK exposes three primary representations:

1. **Raw extrinsics** – encoded byte payloads plus metadata (`EncodedExtrinsic`).
2. **Decoded extrinsics** – call payloads with optional signature (`Extrinsic<T>`).
3. **Signed extrinsics** – decoded extrinsics guaranteed to carry a signature
   (`SignedExtrinsic<T>`).

Understanding the differences helps you pick the correct view when traversing
blocks or analysing transactions.

## 1. Raw Extrinsics (`EncodedExtrinsic`)

Produced by the encoded view (`client.block(id).encoded()`):

```rust
let mut encoded_view = block.encoded();
let encoded = encoded_view.get(0u32, EncodeSelector::Extrinsic).await?;
```

Fields:

- `data: Option<String>` – hex-encoded extrinsic payload (SCALE bytes). When
  `EncodeSelector::None` is used, this will be `None`.
- `metadata: Metadata` – includes block id, extrinsic index,
  pallet id, variant id, and block hash/number used to fetch the extrinsic.
- `signer_payload: Option<SignerPayload>` – contains `ss58_address`, `nonce`,
  `app_id`, and other signing data if the node exposes it.

Operations available on the encoded view (`EncodedExtrinsics`):

- Retrieve a specific extrinsic by index/hash via `get`.
- Retrieve the first/last extrinsic matching filters via `first`/`last`.
- Retrieve the complete set of matching extrinsics via `all`.
- Check existence or count without fetching payloads via `exists`/`count`.

Common use cases:

- Re-encoding or re-submitting raw extrinsics.
- Inspecting signer payload fields without decoding the entire call.
- Counting/existence checks by omitting payload bytes (`EncodeSelector::None`).

Conversion helpers:

- `Extrinsic::<T>::try_from(EncodedExtrinsic)` – decode into a typed call
  (`T: HasHeader + Decode`).
- `SignedExtrinsic::<T>::try_from(EncodedExtrinsic)` – decode into a signed
  extrinsic (fails with `Err(String)` if the payload lacks a signature).

## 2. Decoded Extrinsics (`Extrinsic<T>`)

Produced by the decoded view (`client.block(id).ext()`):

```rust
let mut decoded_view = block.ext();
let maybe_signed = decoded_view.first::<avail::balances::calls::TransferKeepAlive>(Default::default()).await?;
```

Fields and helpers:

- `signature: Option<ExtrinsicSignature>` – present when the extrinsic is signed.
- `call: T` – decoded call struct (requires `T: HasHeader + Decode`).
- `metadata: Metadata` – same metadata as the raw view.
- `events(&Client)` – fetch events tied to this extrinsic.
- `app_id()`, `nonce()`, `tip()`, `ss58_address()` – convenience accessors when a
  signature exists.

Use cases:

- Inspecting call parameters without caring whether the extrinsic was signed.
- Supporting pallets that emit unsigned extrinsics (e.g., inherent system
  extrinsics).
- Converting into `SignedExtrinsic<T>` when a signature is required.

## 3. Signed Extrinsics (`SignedExtrinsic<T>`)

Produced by the signed view (`client.block(id).signed()`):

```rust
let mut signed_view = block.signed();
let signed = signed_view.first::<avail::balances::calls::TransferKeepAlive>(Default::default()).await?;
```

Guarantees and helpers:

- Always contains `signature: ExtrinsicSignature`; unsigned extrinsics produce
  an error during conversion.
- Provides the same helpers as `Extrinsic<T>` plus `signature` is non-`None`.
- Exposes events via `events(&Client)` for convenience.

Use cases:

- Auditing account activity (nonce, tip, signer address).
- Checking transaction success/failure via events.
- Feeding signed extrinsics into analytics or re-broadcast pipelines.

## Choosing the Right Representation

| Requirement                               | Recommended View               |
|-------------------------------------------|--------------------------------|
| Need raw SCALE bytes or signer payload    | `block.encoded()`              |
| Need to decode call parameters            | `block.ext()`                  |
| Need guaranteed signature metadata        | `block.signed()`               |
| Need to check inclusion events            | `block.ext()` or `block.signed()` |
| Need lightweight existence/count checks   | `block.encoded()` (`EncodeSelector::None`)

Illustrative transitions:

- Raw → Decoded call:

```rust
use avail_sdk::block::Extrinsic;
use avail_sdk::avail;

let encoded = block.encoded().get(0u32, EncodeSelector::Extrinsic).await?;
if let Some(encoded) = encoded {
    let decoded: Extrinsic<avail::balances::calls::TransferKeepAlive> = encoded.try_into()?;
    println!("call args: {:?}", decoded.call.value);
}
```

- Decoded → Signed transaction:

```rust
use avail_sdk::block::SignedExtrinsic;

let decoded = block.ext().first::<avail::balances::calls::TransferKeepAlive>(Default::default()).await?;
if let Some(decoded) = decoded {
    let signed: SignedExtrinsic<_> = decoded.try_into()?; // fails if unsigned
    println!("nonce: {}", signed.nonce());
}
```

You can transition between representations:

- Encoded view (`block.encoded()`) → decoded or signed views via `block.ext()` / `block.signed()`.
- `Extrinsic<T>` → `SignedExtrinsic<T>` using `SignedExtrinsic::try_from`
  (fails when unsigned).

Understanding these layers makes it easier to balance performance (raw view)
against ergonomics (decoded view) and accuracy (signed transactions).
