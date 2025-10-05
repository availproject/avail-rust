cargo build

cd zig
zig run -I ./include/ ./src/main.zig -lc ./../target/debug/libavail_rust_ffi.so
