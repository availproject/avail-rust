cargo build

cd c
gcc main.c -lavail_rust_ffi -L./../target/debug
LD_LIBRARY_PATH=./../target/debug ./a.out