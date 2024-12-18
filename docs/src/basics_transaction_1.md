# Transaction - 1

Every transaction consists of the following parts:
- Signature
- Payload
- Transaction Parameters

The Signature defines who is accountable and who's funds will be taken in order to pay for transaction execution.   
The Payload is the function (together with the data) that will be executed.   
The Transaction Parameters define additional information about our transaction. Here we would say how much tip we want to give, what nonce to use, etc.   

In order for our transaction to be executed we need the following parts:
- Establish WebSocket or HTTP connection with a network
- Way to submit a transaction
- Way to check if that transaction was successfully included

## Setting up the stage
Our initial setup will have nothing more than the bare minimum to compile our code.   
Most of the types that we need are included in the `prelude` import collection but because we are not going to use any of it (for now) we will have to manually import modules.

All the future code that we will write will go inside the `main` function.

```rs
use avail_rust::error::ClientError;

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    // Code goes here

    Ok(())
}
```

## Connection
The first thing that we need to do is to establish a connection with an existing network. For the sake of brevity, we will cover only how to do it using websockets but you can find in other examples on how to do it either using HTTP or a custom solution.


```rs
{{#include ./../../examples/docs/basics_1/main.rs:connection}}
```

`rpc_client` is a low level API that allows us to communicate with our network via rpc calls.   
`online_client` is a higher level API that provides many helper functions and abstractions.

## Accounts
> An account represents an identity—usually of a person or an organization—that is capable of making transactions or holding funds.
> In general, every account has an owner who possesses a public and private key pair. The private key is a cryptographically-secure sequence of randomly-generated numbers. For human readability, the private key generates a random sequence of words called a secret seed phrase or mnemonic.  
> [Substrate - Accounts, Addresses, Keys](https://docs.substrate.io/learn/accounts-addresses-keys/)

To create an account we paste our secret seed as an argument to `SecretUri` and then pass that `Keypair`. In this case, we will use the default development account named `Alice`.   
In production you would pass your secret seed via env variable or read it from file.

For Bob use `//Bob`, for Eve use `//Eve`, etc.

```rs
{{#include ./../../examples/docs/basics_1/main.rs:accounts}}
```

## Payload
Payload defines what operation will be executed on the chain. Payload consists of three components:
- Pallet Name
- Call Name
- Call Data

What you need to know is that all the payloads are defines in the following path `avail_rust::avail::*::calls::types::**;` where `*` represents the pallet name and `**` represents the call type.   
For more examples go to the next page.

```rs
{{#include ./../../examples/docs/basics_1/main.rs:payload}}
```

## Transaction Parameters
There are four transaction parameters:
- nonce
- app_id
- tip
- mortality

Manually building the transaction parameters is a tedious and convoluted job so here we are using a helper object to do that for us.  
With the `Options` object we can set what parameters we want to use and with calling `build()` it populates all the non-set params with default values.   
Here are the default values for all the parameters:
- nonce: It uses the best block nonce and it increments it if there are existing transaction in the tx pool with the same nonce
- app_id: 0
- tip: 0
- mortality: The transaction will be alive for 32 blocks starting from current best block hash(height)

```rs
{{#include ./../../examples/docs/basics_1/main.rs:params}}
```

## Signature
Adding signature to an existing payload and transaction params allows us to build an transaction that is ready to be submitted.

```rs
{{#include ./../../examples/docs/basics_1/main.rs:signature}}
```

## Submission
Submission is done by calling `.submit()`. There is another method available as well, `.submit_and_watch()`, but that one isn't working correctly.   
Submitting a transaction yields back the transaction hash.

```rs
{{#include ./../../examples/docs/basics_1/main.rs:submission}}
```


## Watcher
Just because we have submitted our transaction it doesn't mean it was successful or  that it got executed at all.   
We need to implement a `watcher` that will check the next N blocks to see if our tx hash is included in the block.

Once found, we need to search for the `ExtrinsicSuccess` event in order to determine if the transaction was successful or not.

```rs
{{#include ./../../examples/docs/basics_1/main.rs:watcher}}
```


## Source Code 
```rs
{{#include ./../../examples/docs/basics_1/main.rs}}
```