#!/usr/bin/env ruby
# frozen_string_literal: true

require 'fiddle/import'

module AvailFFI
  extend Fiddle::Importer

  DLL_PATH = File.expand_path('../target/debug/libavail_rust_ffi.so', __dir__)
  dlload DLL_PATH

  extern 'void hello_from_rust(void)'
  extern 'void initialize_client(const char *endpoint)'
  extern 'int initialize_signer(const char *seed)'
  extern 'int do_submit_data(int signer_handle, const char *data, int app_id)'
  extern 'int get_transaction_receipt(int submitted_tx_handle)'
  extern 'Receipt *receipt_new(int receipt_handle)'
  extern 'void receipt_free(Receipt *ptr)'

  Receipt = struct [
    'int block_height',
    'unsigned char block_hash[32]',
    'int transaction_index',
    'unsigned char transaction_hash[32]',
    'int handle',
  ]
end

def hex_from_bytes(buffer, len)
  (0...len).map { |i| format('%02x', buffer[i]) }.join
end


AvailFFI.initialize_client('https://turing-rpc.avail.so/rpc')
signer_handle = AvailFFI.initialize_signer('bottom drive obey lake curtain smoke basket hold race lonely fit walk')
submitted_tx_handle = AvailFFI.do_submit_data(signer_handle, 'Hello from Ruby', 2)
receipt_handle = AvailFFI.get_transaction_receipt(submitted_tx_handle)

ptr = AvailFFI.receipt_new(receipt_handle)
raise 'receipt_new returned NULL' if ptr.null?

receipt = AvailFFI::Receipt.new(ptr)
summary = {
  block_height: receipt['block_height'],
  transaction_index: receipt['transaction_index'],
  block_hash: hex_from_bytes(receipt['block_hash'], 32),
  transaction_hash: hex_from_bytes(receipt['transaction_hash'], 32),
}
AvailFFI.receipt_free(ptr)

puts "Ruby: Block Height #{summary[:block_height]}, Tx Index #{summary[:transaction_index]}"
puts "Ruby: Block Hash 0x#{summary[:block_hash]}"
puts "Ruby: Tx Hash 0x#{summary[:transaction_hash]}"
