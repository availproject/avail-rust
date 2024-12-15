#!/bin/bash
set -x
set -e

# SDK
cargo check
cargo check --no-default-features --features "native"
cargo check --target wasm32-unknown-unknown --no-default-features --features "wasm"

# Examples
cd ./examples
cargo check
cargo check --no-default-features
