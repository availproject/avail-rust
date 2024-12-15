#!/bin/bash
set -x
set -e

cargo check
cargo check --no-default-features --features "native"
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm"

# Book
cd ./docs/book
cargo check
cargo check --no-default-features

# Extrinsics
cd ../extrinsics
cargo check
cargo check --no-default-features
