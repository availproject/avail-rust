CRATE="$1"
OP="$2"

if [ "$CRATE $OP" == "core timings" ]; then
    cd ./core
    cargo build --timings
fi

if [ "$CRATE $OP" == "core lines" ]; then
    cargo llvm-lines -p avail-rust-core > core_lines.txt
fi

if [ "$CRATE $OP" == "client timings" ]; then
    cd ./client
    cargo build --timings
fi

if [ "$CRATE $OP" == "core lines" ]; then
    cargo llvm-lines -p avail-rust-client > client_lines.txt
fi
