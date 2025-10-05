local ffi = require("ffi")

local lib_path = "../target/debug/libavail_rust_ffi.so"
local rust = ffi.load(lib_path)

ffi.cdef[[
typedef struct {
  int block_height;
  unsigned char block_hash[32];
  int transaction_index;
  unsigned char transaction_hash[32];
  int handle;
} Receipt;

void hello_from_rust(void);
void initialize_client(const char *endpoint);
int initialize_signer(const char *seed);
int do_submit_data(int signer_handle, const char *data, int app_id);
int get_transaction_receipt(int submitted_tx_handle);
Receipt *receipt_new(int receipt_handle);
void receipt_free(Receipt *ptr);
]]

local function hex_from_buffer(buf, len)
  local out = {}
  for i = 0, len - 1 do
    out[#out + 1] = string.format("%02x", buf[i])
  end
  return table.concat(out)
end

local function inspect_receipt(handle)
  local receipt_ptr = rust.receipt_new(handle)
  assert(receipt_ptr ~= nil, "receipt_new returned nil")

  local receipt = receipt_ptr[0]
  local summary = {
    block_height = receipt.block_height,
    transaction_index = receipt.transaction_index,
    block_hash = hex_from_buffer(receipt.block_hash, 32),
    transaction_hash = hex_from_buffer(receipt.transaction_hash, 32),
  }

  rust.receipt_free(receipt_ptr)
  return summary
end

print("Lua: hello_from_rust() ->")
rust.hello_from_rust()

rust.initialize_client("https://turing-rpc.avail.so/rpc")
local signer_handle = rust.initialize_signer("bottom drive obey lake curtain smoke basket hold race lonely fit walk")
local submitted_tx_handle = rust.do_submit_data(signer_handle, "Hello from Lua", 2)
local receipt_handle = rust.get_transaction_receipt(submitted_tx_handle)
local receipt = inspect_receipt(receipt_handle)
print(string.format("Lua: Block Height %d, Tx Index %d", receipt.block_height, receipt.transaction_index))
print("Lua: Block Hash 0x" .. receipt.block_hash)
print("Lua: Tx Hash 0x" .. receipt.transaction_hash)

