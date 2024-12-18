# Transaction - 3
We are not done yet.

## Setting up the stage
Here we are using the prelude import to import all the necessary type declarations.

```rs
use avail_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // Code goes here

    Ok(())
}
```

## Connection
Both online_client and rpc_client can be grouped together and now we are using a wrapper to get the local node endpoint instead of writing it manually.   
Together with `local_endpoint` we have the following other endpoints:
- `local_http_endpoint`
- `turing_endpoint`
- `turing_http_endpoint`
- `mainnet_endpoint`
- `mainnet_http_endpoint`

```rs
{{#include ./../../examples/docs/basics_3/main.rs:connection}}
```

`new()` method will use a reconnecting websocket rpc. If just HTTP is needed then `new_http()` will do the trick otherwise for custom clients `new_custom(online_client: AOnlineClient, rpc_client: RpcClient)` can be used.

## Payload
Payload creation has been abstracted away and can now be accessed via `sdk.tx.*.**` path where `*` is pallet name and `**` is call name. Not all transaction have been abstracted away and for some you will still need to use the following path `avail_rust::avail::tx().*().**()`. 

```rs
{{#include ./../../examples/docs/basics_3/main.rs:payload}}
```

The object that is returned by the interface has many different helper functions attach to it. Make sure to check them out. 

For some of them you will need to import either `avail_rust::transaction::WebSocket` or `avail_rust::transaction::HTTP` in order to get access to the desired `execute_*` call.   
The prelude import automatically imports the `WebSocket` one and if that's not desired then you will need avoid prelude import and manually import all the type declarations.

## Transaction Parameters, Signature, Submission, Watcher
The watcher is now combined with transaction submission. We can now choose if we want to wait for block inclusion, block finalization, and or fire and forget.   
If we choose to wait, the system will automatically resubmit our transaction in case it didn't found it the next X blocks by using the same transaction parameters.   
WebSocket and HTTP interface differ here in types and implementation:
- WebSocket uses block subscription to fetch blocks.
- HTTP cannot use block subscription because that's a websocket features thus we are forced to fetch headers every N (set to 3 by default) seconds in order to know if a new block has been produced.

The default 3 seconds (sleep_duration) for HTTP can be configured by calling `execute_and_watch` instead of `execute_and_watch_inclusion` or `execute_and_watch_finalization`.   

```rs
{{#include ./../../examples/docs/basics_3/main.rs:signsend}}
```


## Source Code 
```rs
{{#include ./../../examples/docs/basics_3/main.rs}}
```