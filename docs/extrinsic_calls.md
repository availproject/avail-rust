# ExtrinsicCalls Helper

`ExtrinsicCalls` provides a read-only view over decoded call payloads for a single block. It owns a
`BlockContext` to scope RPC requests and accepts optional filters via `ExtrinsicsOpts`.

```
                                     ExtrinsicCalls
+--------------------------------------+-----------------------------------------------+
| Internal State                       | Public Interface                              |
+--------------------------------------+-----------------------------------------------+
| ctx: BlockContext (owns RPC context) | new(client, block_id)                         |
|                                      | get<T>(extrinsic_id)                          |
|                                      | first<T>(opts)                                |
|                                      | last<T>(opts)                                 |
|                                      | all<T>(opts)                                  |
|                                      | count<T>(opts)                                |
|                                      | exists<T>(opts)                               |
|                                      | set_retry_on_error(value)                     |
|                                      | should_retry_on_error()                       |
+--------------------------------------+-----------------------------------------------+
                 consumes                                   uses
                   |                                         |
                   v                                         v
         ExtrinsicsOpts (filter builder)         Client, HashStringNumber, EncodeSelector,
                                                rpc::ExtrinsicOpts, T: HasHeader + Decode
```

The companion helpers share the same block-scoped pattern:

- `EncodedExtrinsics` fetches raw SCALE payloads plus metadata.
- `Extrinsics` decodes raw payloads into runtime calls with optional signatures.
- `SignedExtrinsics` filters decoded extrinsics down to those carrying signatures.

## EncodedExtrinsic

```
                                   EncodedExtrinsic
+--------------------------------------+-----------------------------------------------+
| Fields                               | Methods                                       |
+--------------------------------------+-----------------------------------------------+
| data: String (SCALE payload)         | new(data, metadata, signer_payload)           |
| metadata: Metadata (call context)    | events(client) -> Result<AllEvents>           |
| signer_payload: Option<SignerPayload>| ext_index() -> u32                            |
|                                      | ext_hash() -> H256                            |
|                                      | app_id() -> Option<u32>                       |
|                                      | nonce() -> Option<u32>                        |
|                                      | ss58_address() -> Option<String>              |
|                                      | as_signed<T>() -> Result<SignedExtrinsic<T>>  |
|                                      | as_extrinsic<T>() -> Result<Extrinsic<T>>     |
|                                      | is<T>() -> bool                               |
|                                      | header() -> (u8, u8)                          |
+--------------------------------------+-----------------------------------------------+
                 contains                                    interacts with
                   |                                         |
                   v                                         v
   Metadata (ext_hash, ext_index, pallet_id,       AllEvents/Events, SignedExtrinsic<T>,
   variant_id, block_id; constructor `new`)        Extrinsic<T>, Client, SignerPayload
```
