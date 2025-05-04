#!/bin/bash
set -x
set -e

# SDK
cargo check
cargo check --no-default-features --features "native"
rustup target add wasm32-unknown-unknown
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm"

# Examples
# cd ./examples
# cargo check
# cargo check --no-default-features

