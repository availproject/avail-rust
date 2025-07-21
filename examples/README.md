# Examples
### Account (./account)
- Creating Keypair from mnemonic seed
- Creating Account Id from SS58 address or Keypair
- Displaying the SS58 Address of a Account Id
- Fetching Account Balance, Nonce and Info

### Client (./client)
- Fetching Block Header
- Fetching Block State
- Fetching Block Hash from Block Height
- Fetching Block Height from Block Hash
- Fetching Block Hash and Height

### Block and Block Transactions (./block_client)
- Fetching the whole Block via Block RPC (includes block header, block transactions and block justifications)
- Fetching and filtering Block Transactions via Block Transactions RPC
- Decoding Transactions from both mentioned RPCs

### Transaction Events (./event_client)
- Fetching block and transaction events via event client
- Decoding block and transaction events
- Fetching and Decoding events from historical blocks

### Encoding and Decoding (./encoding_decoding)
- Encoding and Decoding Transactions
- Encoding and Decoding Events
- Encoding and Decoding Storage

### Transaction Submission and Receipt (./transaction_submission)
- Transaction Creation
- Transaction Submission
- Fetching Transaction Receipt
- Fetching Block State
- Fetching Block Transaction
