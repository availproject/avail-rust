# Transaction - 2

With everything in place, we can slowly replace and tidy up our code.

## Connection
The first change that we have does was to simplify the creation of online and rpc client. Instead of manually creating them there is an convenient helper function that will set it up for us.   

```rs
{{#include ./../../examples/docs/basics_2/main.rs:connection}}
```

The  `reconnecting_api` create an rpc with the following parameters:

```rs
ReconnectingRpcClient::builder().retry_policy(ExponentialBackoff::from_millis(1000).max_delay(Duration::from_secs(3)).take(3))
```

## Accounts
There are already premade accounts available in the SDK interface. There is one as well for Bob, Eve, and Charlie.

```rs
{{#include ./../../examples/docs/basics_2/main.rs:accounts}}
```

## Payload
Manually passing the pallet name, call name and call data is error prone and that's why there is an better way.   
All the payloads are defined in the following path `avail_rust::avail::tx().*().**(data)` where `*` is the pallet name and `**` is the call type.   
For more examples go to the next page.

```rs
{{#include ./../../examples/docs/basics_2/main.rs:payload}}
```

## Transaction Parameters, Signature, Submission
Transaction parameters, signature, and submission can be combined all into one single call.   
Because we are using the default transaction parameters, we are passing `None` as the argument. If we wish to alter the parameters, we would pass an optional `Options` object.

```rs
{{#include ./../../examples/docs/basics_2/main.rs:signsend}}
```

## Watcher
Just like the rest, the watching part can be abstracted as well.   
Finding if a transaction was successful or not is now just a matter of calling `is_successful()`. If the transaction failed, it will return an error with the description on why it failed.   
The last arguments, `Some(3)`, tells the watcher to read the next 4 (this is not a typo, it's X + 1) blocks and if none of the contains the target transaction hash it will return an error.

```rs
{{#include ./../../examples/docs/basics_2/main.rs:watcher}}
```


## Source Code 
```rs
{{#include ./../../examples/docs/basics_2/main.rs}}
```