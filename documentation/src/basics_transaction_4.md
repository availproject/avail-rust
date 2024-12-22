# Transaction - 4

The SDK offer much more than just a couple of helper functions for transaction submission. To get a better understanding on what can be done, check all the other examples, especially the following ones:
- `Data Submission` - This one is a must
- `Block`
- `Transactions`
- `Events`

Not everything is shown in our examples. Open the SDK and take a look what interfaces are available.

## Events and Block
Here is just a sneak peak on the events and block api that we offer.

```rs
{{#include ./../../examples/docs/basics_4/main.rs:event}}
```

```rs
{{#include ./../../examples/docs/basics_4/main.rs:block}}
```

## Transaction with custom payload
Transaction interface can be created using custom payload.

```rs
{{#include ./../../examples/docs/basics_4/main.rs:custompayload}}
```

## Source Code 
```rs
{{#include ./../../examples/docs/basics_4/main.rs}}
```