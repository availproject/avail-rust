
from cffi import FFI
ffi = FFI()

ffi.cdef("""
    typedef struct
    {
        int block_height;
        unsigned char block_hash[32];
        int transaction_index;
        unsigned char transaction_hash[32];
        int handle;
    } Receipt;

    void initialize_client(char *);

    // Params: Secret Seed
    // Returns: Signer Handle
    int initialize_signer(char *);

    // Params: Signer Handle, Data (as string), App Id
    // Returns: Submitted Transaction Handle
    int do_submit_data(int, char *, int);

    // Params: Submitted Transaction Handle
    // Returns: Transaction Receipt Handle
    int get_transaction_receipt(int);

    // Params: Transaction Receipt Handle
    // Returns: Rust Allocated Receipt struct
    // Note: Make sure to call receipt_free!
    Receipt *receipt_new(int);

    // Params: Rust Allocated Receipt struct
    void receipt_free(Receipt *);
""")
avail = ffi.dlopen("./../target/debug/libavail_rust_ffi.so")

endpoint = ffi.new("char[]", b"https://turing-rpc.avail.so/rpc")
avail.initialize_client(endpoint)

signer_seed = ffi.new("char[]", b"bottom drive obey lake curtain smoke basket hold race lonely fit walk")
signer_handle = avail.initialize_signer(signer_seed)

data = ffi.new("char[]", b"Hello from Python")
submitted_handle = avail.do_submit_data(signer_handle, data, 2)

receipt_handle = avail.get_transaction_receipt(submitted_handle)
receipt = avail.receipt_new(receipt_handle)
block_hash = bytes(ffi.buffer(receipt.block_hash)).hex()
transaction_hash = bytes(ffi.buffer(receipt.transaction_hash)).hex()

print(
    "Python: Block Height: "
    + str(receipt.block_height)
    + ", Transaction Index: "
    + str(receipt.transaction_index)
    + ", Block Hash: 0x"
    + block_hash
    + ", Transaction Hash: 0x"
    + transaction_hash
)

avail.receipt_free(receipt)
