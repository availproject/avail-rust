const std = @import("std");
const c = @cImport({
    @cInclude("main.h");
});

fn initialize_client(str: [:0]const u8) void {
    c.initialize_client(str);
}

fn initialize_signer(str: [:0]const u8) c_int {
    return c.initialize_signer(str);
}

fn do_submit_data(handle: c_int, str: [:0]const u8, app_id: c_int) c_int {
    return c.do_submit_data(handle, str, app_id);
}

fn get_transaction_receipt(handle: c_int) c_int {
    return c.get_transaction_receipt(handle);
}

fn receipt_new(handle: c_int) [*c]c.Receipt {
    return c.receipt_new(handle);
}

fn receipt_free(receipt: [*c]c.Receipt) void {
    c.receipt_free(receipt);
}

pub fn main() !void {
    c.hello_from_rust();

    // const a = "https://turing-rpc.avail.so/rpc";
    // const b: [*c]u8 = a.*;
    // c.initialize_client(b);

    initialize_client("https://turing-rpc.avail.so/rpc");
    const signer_handle = initialize_signer("bottom drive obey lake curtain smoke basket hold race lonely fit walk");
    const submitted_tx_handle = do_submit_data(signer_handle, "Hello from Zig", 2);
    const receipt_handle = get_transaction_receipt(submitted_tx_handle);
    const receipt: [*c]c.Receipt = receipt_new(receipt_handle);
    const block_height = receipt[0].block_height;
    const transaction_index = receipt[0].transaction_index;
    const block_hash = receipt[0].block_hash;
    const transaction_hash = receipt[0].transaction_hash;

    // print straight to stderr
    std.debug.print("Block Hash: 0x{s}, Height: {d}, Transaction Hash: 0x{s}, Index: {d}\n", .{ std.fmt.fmtSliceHexLower(&block_hash), block_height, std.fmt.fmtSliceHexLower(&transaction_hash), transaction_index });

    receipt_free(receipt);
}
